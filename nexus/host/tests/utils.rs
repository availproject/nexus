use anyhow::anyhow;
use httpmock::prelude::*;
use httpmock::Mock;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub fn get_mock_server() -> MockServer {
    MockServer::start()
}

pub fn add_check_body_and_response<'a>(mock_server: &'a MockServer, body_contains: &'a str, response: &'a str) -> Mock<'a> {
    mock_server.mock(|when, then| {
        when.method(POST).path("/").json_body_partial(
            r#"
                {
                    "method": "eth_getBlockByNumber"
                }
            "#,
        );
        then.status(200).body(response);
    })
}

pub fn run_nexus_client() -> anyhow::Result<()> {
    let target_directory = "../../nexus/host";

    // Check if we are in the directory
    if !Path::new(&target_directory).is_dir() {
        return Err(anyhow!("Directory not found: {}", target_directory));
    }

    let mut command = Command::new("cargo");
    command.current_dir(target_directory).stdout(Stdio::piped()).stderr(Stdio::piped());

    // Running nexus in dev mode
    command.env("RUST_LOG", "info");
    command.env("RISC0_DEV_MODE", "1");
    command.args(&["run", "--no-default-features", "--features", "risc0", "--", "--dev"]);

    println!("Starting client in directory: {}", target_directory);
    println!("Command: RUST_LOG=info RISC0_DEV_MODE=1 cargo run --no-default-features --features \"risc0\" -- --dev");

    let child = command.spawn()?;
    let child_id = monitor_output_for_success(child, "Built client", 600).expect("Failed to run nexus client");

    println!("Client started with PID: {}", child_id);

    Ok(())
}

pub fn run_mock_geth_adapter_once(ethereum_rpc_url: String) -> anyhow::Result<()> {
    let target_directory = "../../examples/mock_geth_adapter/host";

    // Check if we are in the directory
    if !Path::new(&target_directory).is_dir() {
        return Err(anyhow!("Directory not found: {}", target_directory));
    }

    let mut command = Command::new("cargo");
    command.current_dir(target_directory).stdout(Stdio::piped()).stderr(Stdio::piped());

    // Running nexus in dev mode
    command.env("RUST_LOG", "info");
    command.env("RISC0_DEV_MODE", "1");
    command.args(&["run", "--", &ethereum_rpc_url, "--dev"]);

    println!("Starting client in directory: {}", target_directory);
    println!(
        "Command: RUST_LOG=info RISC0_DEV_MODE=1 cargo run -- \"{}\" --dev",
        ethereum_rpc_url
    );

    let child = command.spawn()?;
    let child_id = monitor_output_for_success(child, "Got header", 600).expect("Failed to run mock geth adapter");

    println!("Client started with PID: {}", child_id);

    Ok(())
}

/// Monitors a child process's output in real-time until a success message is found
/// Returns the process ID without killing the process
fn monitor_output_for_success(mut child: Child, success_message: &str, timeout_secs: u64) -> Result<u32, anyhow::Error> {
    let pid = child.id();

    let stdout = match child.stdout.take() {
        Some(out) => out,
        None => {
            let _ = child.kill();
            return Err(anyhow!(
                "Failed to capture stdout - stdout wasn't properly piped"
            ));
        }
    };

    let stderr = match child.stderr.take() {
        Some(err) => err,
        None => {
            let _ = child.kill();
            return Err(anyhow!(
                "Failed to capture stderr - stderr wasn't properly piped"
            ));
        }
    };

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let start_time = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    let stdout_success_message = success_message.to_string();
    let stdout_handle = std::thread::spawn( move || {
        for line in stdout_reader.lines() {
            match line {
                Ok(content) => {
                    println!("[stdout] {}", content);
                    if content.contains(&stdout_success_message) {
                        println!("✓ Success message found: '{}'", &stdout_success_message);
                        return Ok(());
                    }
                }
                Err(e) => {
                    return Err(anyhow!("Error reading stdout: {}", e));
                }
            }
        }
        Err(anyhow!("stdout closed without finding success message"))
    });

    let stderr_handle = std::thread::spawn(move || {
        for line in stderr_reader.lines() {
            match line {
                Ok(content) => {
                    println!("[stderr] {}", content);
                }
                Err(e) => {
                    return Err(anyhow!("Error reading stderr: {}", e));
                }
            }
        }
        Ok(())
    });

    loop {
        if start_time.elapsed() > timeout {
            let _ = child.kill();
            return Err(anyhow!(
                "Timeout after {} seconds - success message not found",
                timeout_secs
            ));
        }

        if stdout_handle.is_finished() {
            match stdout_handle.join() {
                Ok(result) => match result {
                    Ok(_) => return Ok(pid),
                    Err(e) => {
                        let _ = child.kill();
                        return Err(e);
                    }
                },
                Err(_) => {
                    let _ = child.kill();
                    return Err(anyhow!("Stdout monitoring thread panicked"));
                }
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}
