pub use avail_subxt::Header;
use nexus_core::{state_machine::StateMachine, zkvm::ProverMode};

#[cfg(any(feature = "risc0"))]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};

use host::{run_nexus, setup_components};
#[cfg(any(feature = "sp1"))]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};
pub use relayer::{Relayer, SimpleRelayer};
use std::env::args;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let dev_flag = args.iter().any(|arg| arg == "--dev");
    if dev_flag {
        info!("⚠️  Running in dev mode - proofs are not valid");
    }

    let prover_mode = if dev_flag {
        ProverMode::MockProof
    } else {
        ProverMode::Compressed
    };

    print_animated_logo(&prover_mode);

    let (node_db, state) = setup_components("./db");
    let mut state_machine = StateMachine::<ZKVM, Proof>::new(state.clone());

    let avail_rpc = args
        .iter()
        .find(|arg| arg.starts_with("--avail-rpc="))
        .map(|arg| arg.trim_start_matches("--avail-rpc="))
        .unwrap_or("wss://turing-rpc.avail.so:443/ws");

    info!("Connecting to Avail RPC at: {}", avail_rpc);
    let relayer_mutex = Arc::new(Mutex::new(SimpleRelayer::new(avail_rpc)));
    // Shared shutdown signal using a watch channel
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Create a Tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Run the tasks inside the runtime
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

        // Spawn the main Nexus logic
        info!("Starting execution engine");
        let nexus_task = tokio::spawn(async move {
            run_nexus(
                relayer_mutex,
                node_db,
                state_machine,
                (prover_mode, 7000),
                state,
                shutdown_rx,
            )
            .await;
        });

        // Wait for both tasks to complete
        if let Err(e) = tokio::try_join!(shutdown_task, nexus_task) {
            error!("Error during execution: {:?}", e);
        }
    });

    info!("Nexus node stopped");
    Ok(())
}

fn print_animated_logo(prover_mode: &ProverMode) {
    let version_line = format!("                                    ║           Version: {:8}       ║                                    ", env!("CARGO_PKG_VERSION"));
    let mode_line = format!("                                    ║      Prover Mode: {:12}    ║                                    ", format!("{:?}", prover_mode));

    let logo = vec![
        "                                    ╔═══════════════════════════════════╗                                    ",
        "                                    ║            NEXUS NODE             ║                                    ",
        "                                    ╚═══════════════════════════════════╝                                    ",
        "                                                                                                            ",
        "     ███╗   ██╗███████╗██╗  ██╗██╗   ██╗███████╗    ███╗   ██╗ ██████╗ ██████╗ ███████╗                 ",
        "     ████╗  ██║██╔════╝╚██╗██╔╝██║   ██║██╔════╝    ████╗  ██║██╔═══██╗██╔══██╗██╔════╝                 ",
        "     ██╔██╗ ██║█████╗   ╚███╔╝ ██║   ██║███████╗    ██╔██╗ ██║██║   ██║██║  ██║█████╗                   ",
        "     ██║╚██╗██║██╔══╝   ██╔██╗ ██║   ██║╚════██║    ██║╚██╗██║██║   ██║██║  ██║██╔══╝                   ",
        "     ██║ ╚████║███████╗██╔╝ ██╗╚██████╔╝███████║    ██║ ╚████║╚██████╔╝██████╔╝███████╗                 ",
        "     ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝    ╚═╝  ╚═══╝ ╚═════╝ ╚═════╝ ╚══════╝                 ",
        "                                                                                                            ",
        "                                    ╔═══════════════════════════════════╗                                    ",
        &version_line,
        &mode_line,
        "                                    ╚═══════════════════════════════════╝                                    ",
    ];

    for line in logo {
        println!("{}", line);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Log the startup (this will go to winston/other loggers)
    info!(
        "Nexus node started - Version: {} Mode: {:?}",
        env!("CARGO_PKG_VERSION"),
        prover_mode
    );
}
