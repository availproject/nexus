use host::instrumentation::Instrumentation;
use host::{get_latest_proven_block, prover_handle};
use nexus_core::db::SharedDB;
use nexus_core::metrics::ProvingMetrics;
use nexus_core::zkvm::ProverMode;
use std::env::args;
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{error, info};
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("nexus=info".parse().unwrap())
                .add_directive("info".parse().unwrap()),
        )
        .with_thread_names(false)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .with_ansi(true)
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .init();

    // Setup Instrumentation
    let mut analytics = Instrumentation::new("nexus-prover".to_string());
    analytics.setup()?;

    let args: Vec<String> = args().collect();
    let postgres_database_url = args
        .clone()
        .iter()
        .find(|arg| arg.starts_with("--database-url="))
        .map(|arg| arg.trim_start_matches("--database-url="))
        .unwrap_or("postgres://user:password@localhost:5432/db_name")
        .to_string();
    let start_block = args
        .clone()
        .iter()
        .find(|arg| arg.starts_with("--start-block="))
        .map(|arg| arg.trim_start_matches("--start-block="))
        .unwrap_or("0")
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("Invalid start-block value: must be a valid u32"))?;
    let dev_flag = args.iter().any(|arg| arg == "--dev");
    if dev_flag {
        info!("⚠️  Running in dev mode - proofs are not valid");
    }
    let prover_mode = if dev_flag { ProverMode::MockProof } else { ProverMode::Compressed };

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Create a Tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        // Spawn a task to handle the Ctrl+C signal
        let shutdown_task = tokio::spawn(async move {
            if let Err(err) = tokio::signal::ctrl_c().await {
                error!("Failed to listen for shutdown signal: {:?}", err);
            } else {
                info!("Received shutdown signal, initiating graceful shutdown");
                let _ = shutdown_tx.send(true); // Notify other components
            }
        });

        let shared_db = Arc::new(SharedDB::init(postgres_database_url).await.expect("Failed to init database."));
        let latest_proven_block = {
            let block = get_latest_proven_block(&shared_db).await.expect("Initial shared DB call failed.");
            if block < start_block as u64 {
                start_block
            } else {
                block as u32
            }
        };

        let proving_metrics = ProvingMetrics::init();

        info!("Starting prover engine with block {}", latest_proven_block);
        let prover_task = tokio::spawn(async move {
            prover_handle(
                shared_db.clone(),
                shutdown_rx,
                latest_proven_block,
                prover_mode,
                proving_metrics,
            )
            .await
        });

        // Wait for both tasks to complete
        if let Err(e) = tokio::try_join!(shutdown_task, prover_task) {
            error!("Error during execution: {:?}", e);
        }
    });

    info!("Prover stopped.");
    Ok(())
}
