use host::prover_handle;
use nexus_core::db::SharedDB;
use nexus_core::zkvm::ProverMode;
use std::sync::Arc;
use std::{env::args, time::Duration};
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::{error, info, warn};
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

const MAX_RETRIES: u32 = 2;
const RETRY_DELAY_SECS: u64 = 5;

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

    let args: Vec<String> = args().collect();
    let postgres_database_url = args
        .clone()
        .iter()
        .find(|arg| arg.starts_with("--database-url="))
        .map(|arg| arg.trim_start_matches("--database-url="))
        .unwrap_or("postgres://user:password@localhost:5432/db_name")
        .to_string();
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
        let latest_proven_block = get_latest_proven_block(&shared_db).await;

        info!("Starting prover engine");
        let prover_task = tokio::spawn(async move {
            prover_handle(
                shared_db.clone(),
                shutdown_rx,
                latest_proven_block as u32,
                prover_mode,
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

async fn get_latest_proven_block(shared_db: &Arc<SharedDB>) -> u64 {
    let mut retries = 0;
    loop {
        sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
        match shared_db
            .get_latest_proven_block()
            .await
            .expect("Unable to fetch latest proven block. Some DB error.")
        {
            Some(block) => {
                return (block + 1).try_into().unwrap();
            }
            None => {
                if retries == MAX_RETRIES {
                    info!("Max retries reached. Starting Proving from block 0");
                    return 0;
                }
                warn!("Didn't found any last proven block. Retrying.....");
                retries += 1;
                continue;
            }
        }
    }
}
