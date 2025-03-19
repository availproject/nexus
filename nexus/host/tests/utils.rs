use anyhow::anyhow;
use httpmock::prelude::*;
use httpmock::Mock;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

// ================================================
// Managed Process
// It will be dropped when test run is finished
pub struct ManagedProcess {
    child: Child,
}

impl ManagedProcess {
    pub fn new(child: Child) -> Self {
        Self { child }
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }
}

impl Drop for ManagedProcess {
    fn drop(&mut self) {
        let pid = self.child.id();
        println!("Automatically killing process with PID: {}", pid);
        if let Err(e) = self.child.kill() {
            println!("Failed to kill process {}: {}", pid, e);
        }
    }
}

// ================================================


pub fn get_mock_server() -> MockServer {
    MockServer::start()
}

pub fn add_check_body_and_response<'a>(mock_server: &'a MockServer, body_contains: &'a str, response: &'a str) -> Mock<'a> {
    mock_server.mock(|when, then| {
        when.method(POST).path("/").json_body_partial(body_contains);
        then.status(200).body(response);
    })
}

pub async fn run_nexus_client() -> anyhow::Result<ManagedProcess> {
    let target_binary = "../../target/debug/host";

    // Check if we are in the directory
    if !Path::new(&target_binary).is_file() {
        return Err(anyhow!(
            "binary not found: {}, run cargo build for nexus client",
            target_binary
        ));
    }

    let mut command = Command::new(target_binary);

    // Running nexus in dev mode
    command.env("RUST_LOG", "info");
    command.env("RISC0_DEV_MODE", "1");
    command.args(&["--dev", "--avail-rpc=ws://127.0.0.1:9944", "--nexus-start-block=1"]);

    let child = command.spawn()?;
    println!("Client started with PID: {}", child.id());

    let managed_process = ManagedProcess::new(child);

    sleep(Duration::from_secs(10)).await;

    Ok(managed_process)
}

pub async fn run_mock_geth_adapter_once(ethereum_rpc_url: String) -> anyhow::Result<ManagedProcess> {
    let target_binary = "../../target/debug/geth-adapter-host";

    // Check if we are in the directory
    if !Path::new(&target_binary).is_file() {
        return Err(anyhow!(
            "binary not found: {}, run cargo build for geth adapter",
            target_binary
        ));
    }

    let mut command = Command::new(target_binary);

    // Running nexus in dev mode
    command.env("RUST_LOG", "info");
    command.env("RISC0_DEV_MODE", "1");
    command.args(&[ethereum_rpc_url, "--dev".into()]);

    let child = command.spawn()?;

    println!("Client started with PID: {}", child.id());

    let managed_process = ManagedProcess::new(child);

    Ok(managed_process)
}
