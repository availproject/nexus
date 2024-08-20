use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::path::Path;
use std::process::{exit, Command as Cmd};

/// Simple CLI to manage nexus services
#[derive(Parser, Debug)]
#[command(version, about = "A simple CLI for managing repository tasks", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Deletes the **/db/ in the specified directory
    Clean {
        #[command(subcommand)]
        clean_cmd: Option<CleanCommands>,
    },

    /// Runs nexus/host
    Nexus,

    /// Runs example/zksync_adapter/host with an optional API URL
    Zksync {
        /// Optional URL for zksync_proof_api
        #[arg(long, default_value_t = String::from("http://127.0.0.1:3030"))]
        url: String,
    },

    /// Sets default environment if any
    Init {
        /// Optional environment name
        #[arg(short, long)]
        env: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum CleanCommands {
    /// Cleans the database in nexus/host
    Nexus,

    /// Cleans the database in examples/zksync_adapter/host
    Zksync,

    /// Cleans the databases in both nexus/host and examples/zksync_adapter/host
    All,
}

fn main() {
    let args = Args::parse();

    // Read the project root from the environment variable
    let project_root =
        env::var("PROJECT_ROOT").expect("PROJECT_ROOT environment variable is not set");
    let nexus_dir = Path::new(&project_root).join("nexus/host");
    let zksync_dir = Path::new(&project_root).join("examples/zksync_adapter/host");

    match args.command {
        Commands::Clean { clean_cmd } => {
            let command = clean_cmd.unwrap_or(CleanCommands::All); // Default to All if None
            match command {
                CleanCommands::Nexus => {
                    if clean_db(&nexus_dir).is_err() {
                        exit(1);
                    }
                }
                CleanCommands::Zksync => {
                    if clean_db(&zksync_dir).is_err() {
                        exit(1);
                    }
                }
                CleanCommands::All => {
                    let nexus_result = clean_db(&nexus_dir);
                    let zksync_result = clean_db(&zksync_dir);

                    // Check if either of the operations failed
                    if nexus_result.is_err() || zksync_result.is_err() {
                        eprintln!("One or more clean operations failed.");
                        exit(1);
                    }
                }
            }
        }
        Commands::Nexus => run_nexus(&nexus_dir),
        Commands::Zksync { url } => run_zksync(&url, &zksync_dir),
        Commands::Init { env } => init_env(env),
    }
}

fn clean_db(dir: &std::path::Path) -> Result<(), std::io::Error> {
    println!("Cleaning the database at {:?}", dir);

    let db_path = dir.join("db");

    if fs::remove_dir_all(&db_path).is_ok() {
        println!("Database directory deleted successfully at {:?}", db_path);
        Ok(())
    } else {
        eprintln!(
            "Failed to delete the database directory at {:?} or it does not exist.",
            db_path
        );
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to delete database directory",
        ))
    }
}

fn run_zksync(api_url: &str, zksync_dir: &Path) {
    println!("Running zksync commands with API URL: {}", api_url);

    // Run the zksync process with the provided URL
    let status_zksync = Cmd::new("cargo")
        .arg("run")
        .arg("--")
        .arg(api_url) // Pass the URL as an argument
        .current_dir(zksync_dir)
        .status()
        .expect("Failed to execute zksync run");

    if !status_zksync.success() {
        eprintln!("Failed to run zksync at {:?}", zksync_dir);
        exit(1);
    }
}

fn run_nexus(nexus_dir: &Path) {
    println!("Running nexus at {:?}", nexus_dir);

    let status = Cmd::new("cargo")
        .arg("run")
        .current_dir(nexus_dir)
        .status()
        .expect("Failed to execute `cargo run`");

    if !status.success() {
        eprintln!("`cargo run` failed with exit status: {}", status);
        exit(1);
    }
}

fn init_env(env: Option<String>) {
    match env {
        Some(env_name) => {
            println!("Not implemented for {} environment yet", env_name);
        }
        None => {
            println!("No environment specified. Using default settings.");
        }
    }
}
