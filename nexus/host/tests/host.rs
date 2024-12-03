use anyhow::Error;
use avail_subxt::config::Header as HeaderTrait;
use avail_subxt::Header;
use host::{run_nexus, setup_components};
use mockall::predicate::*;
use mockall::*;
#[cfg(any(feature = "risc0"))]
use nexus_core::zkvm::risczero::{RiscZeroProof as Proof, RiscZeroProver as Prover, ZKVM};
#[cfg(any(feature = "sp1"))]
use nexus_core::zkvm::sp1::{Sp1Proof as Proof, Sp1Prover as Prover, SP1ZKVM as ZKVM};
use nexus_core::{
    state_machine::StateMachine,
    types::{
        AccountState, AppAccountId, AppId, HeaderStore, InitAccount, StatementDigest,
        TransactionV2, TxParamsV2, TxSignature, H256,
    },
    zkvm::ProverMode,
};
use relayer::Relayer;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{watch, Mutex};
use tokio::task;
use tokio::time::{sleep, Duration};

mock! {
  pub Relayer {}

    impl Relayer for Relayer {
      fn receiver(&mut self) -> Arc<tokio::sync::Mutex<UnboundedReceiver<Header>>>;
      fn get_header_hash(&self, height: u32) -> impl Future<Output = H256> + Send;
      fn start(&self, start_height: u32) -> impl Future<Output = ()> + Send;
      fn stop(&self);
    }

    impl Clone for Relayer {
        fn clone(&self) -> Self;
    }
}

#[tokio::test]
async fn test_empty_batches() {
    use serde_json;
    use tokio::fs;
    // Mock the Relayer instance
    let mut mock_relayer = MockRelayer::new();

    // Set up an unbounded channel to simulate sending and receiving headers
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Header>();

    // Mock `receiver` to return the receiver end of the channel
    let receiver_arc: Arc<Mutex<UnboundedReceiver<Header>>> = Arc::new(Mutex::new(receiver));
    let receiver_arc_clone = receiver_arc.clone();
    let db_path = "./tests/db/test_empty_batches";

    mock_relayer
        .expect_receiver()
        .returning(move || receiver_arc_clone.clone());

    // Read headers from the JSON file
    let json_path = "tests/data/avail_headers.json";
    let file_content = fs::read_to_string(json_path)
        .await
        .expect("Failed to read headers JSON file");
    let headers: Vec<Header> =
        serde_json::from_str(&file_content).expect("Failed to parse headers JSON file");
    let headers_clone = headers.clone();
    let sender_clone = sender.clone();

    let prover_mode = ProverMode::MockProof;
    let (node_db, state) = setup_components(db_path);
    let mut state_machine = StateMachine::<ZKVM, Proof>::new(state.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    mock_relayer.expect_start().returning(move |_| {
        let headers_in_box = headers_clone.clone();
        let sender_in_box = sender_clone.clone();

        Box::pin(async move {
            for header in headers_in_box {
                println!("Mocked start sending header: {:?}", header.clone());

                // Simulate sending headers
                sender_in_box
                    .send(header)
                    .expect("Failed to send header in mock");
            }
        })
    });

    let timeout = tokio::spawn(async move {
        println!("Nexus has shut down gracefully.");

        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("Shutting down nexus");
        shutdown_tx.send(true);
    });

    // Spawn the main Nexus logic
    let nexus_task: tokio::task::JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        run_nexus(
            Arc::new(Mutex::new(mock_relayer)),
            node_db,
            state_machine,
            (prover_mode, 7000),
            state,
            shutdown_rx,
        )
        .await?;

        Ok(())
    });

    let result = tokio::try_join!(nexus_task, timeout);

    match result {
        Ok((nexus_result, _)) => match nexus_result {
            Ok(()) => {
                println!("Nexus ran successfully.");
            }
            Err(e) => {
                panic!("Nexus task failed with error: {:?}", e);
            }
        },
        Err(e) => {
            panic!("Error during shutdown or Nexus operation: {:?}", e);
        }
    }

    // Clean up the database folder
    if let Err(e) = fs::remove_dir_all(db_path).await {
        eprintln!("Failed to clean up database folder: {:?}", e);
    } else {
        println!("Database folder cleaned up successfully.");
    }
}

#[tokio::test]
async fn test_out_of_order_headers() {
    use serde_json;
    use tokio::fs;
    let db_path = "./tests/db/test_out_of_order_headers";

    if let Err(e) = fs::remove_dir_all(db_path.clone()).await {
        eprintln!("Failed to clean up database folder: {:?}", e);
    } else {
        println!("Database folder cleaned up successfully.");
    }

    // Mock the Relayer instance
    let mut mock_relayer = MockRelayer::new();

    // Set up an unbounded channel to simulate sending and receiving headers
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Header>();

    // Mock `receiver` to return the receiver end of the channel
    let receiver_arc: Arc<Mutex<UnboundedReceiver<Header>>> = Arc::new(Mutex::new(receiver));
    let receiver_arc_clone = receiver_arc.clone();

    mock_relayer
        .expect_receiver()
        .returning(move || receiver_arc_clone.clone());

    // Read headers from the JSON file
    let json_path = "tests/data/avail_headers.json";
    let file_content = fs::read_to_string(json_path)
        .await
        .expect("Failed to read headers JSON file");
    let mut headers: Vec<Header> =
        serde_json::from_str(&file_content).expect("Failed to parse headers JSON file");

    // Intentionally shuffle the headers to simulate out-of-order delivery
    headers.reverse();
    let headers_clone = headers.clone();
    let sender_clone = sender.clone();

    let prover_mode = ProverMode::MockProof;
    let (node_db, state) = setup_components(db_path);
    let mut state_machine = StateMachine::<ZKVM, Proof>::new(state.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    mock_relayer.expect_start().returning(move |_| {
        let headers_in_box = headers_clone.clone();
        let sender_in_box = sender_clone.clone();

        Box::pin(async move {
            for header in headers_in_box {
                println!(
                    "Mocked start sending header (out of order): {:?}",
                    header.clone()
                );

                // Simulate sending headers
                sender_in_box
                    .send(header)
                    .expect("Failed to send header in mock");
            }
        })
    });

    let timeout = tokio::spawn(async move {
        println!("Nexus has shut down gracefully.");

        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("Shutting down nexus");
        shutdown_tx.send(true);
    });

    // Spawn the main Nexus logic
    let nexus_task: tokio::task::JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        // The logic should exit or error upon detecting out-of-order headers
        match run_nexus(
            Arc::new(Mutex::new(mock_relayer)),
            node_db.clone(),
            state_machine,
            (prover_mode, 7001),
            state,
            shutdown_rx,
        )
        .await
        {
            Ok(_) => {
                panic!("Nexus did not exit when processing out-of-order headers");
            }
            Err(e) => {
                println!("Nexus exited with error as expected: {:?}", e);
            }
        }

        Ok(())
    });

    let result = tokio::try_join!(nexus_task, timeout);

    match result {
        Ok((nexus_result, _)) => match nexus_result {
            Ok(()) => {
                println!("Test passed: Nexus handled out-of-order headers as expected.");
            }
            Err(e) => {
                panic!("Nexus task failed unexpectedly: {:?}", e);
            }
        },
        Err(e) => {
            panic!("Error during shutdown or Nexus operation: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_state_root_for_empty_batches() {
    use serde_json;
    use tokio::fs;
    let db_path = "./tests/db/test_state_root_for_empty_batches";

    if let Err(e) = fs::remove_dir_all(db_path.clone()).await {
        eprintln!("Failed to clean up database folder: {:?}", e);
    } else {
        println!("Database folder cleaned up successfully.");
    }

    // Mock the Relayer instance
    let mut mock_relayer = MockRelayer::new();

    // Set up an unbounded channel to simulate sending and receiving headers
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Header>();

    // Mock `receiver` to return the receiver end of the channel
    let receiver_arc: Arc<Mutex<UnboundedReceiver<Header>>> = Arc::new(Mutex::new(receiver));
    let receiver_arc_clone = receiver_arc.clone();

    mock_relayer
        .expect_receiver()
        .returning(move || receiver_arc_clone.clone());

    // Read headers from the JSON file
    let json_path = "tests/data/avail_headers.json";
    let file_content = fs::read_to_string(json_path)
        .await
        .expect("Failed to read headers JSON file");
    let mut headers: Vec<Header> =
        serde_json::from_str(&file_content).expect("Failed to parse headers JSON file");

    let headers_clone = headers.clone();
    let sender_clone = sender.clone();

    let prover_mode = ProverMode::MockProof;
    let (node_db, state) = setup_components(db_path);
    let node_db_clone = node_db.clone();
    let mut state_machine = StateMachine::<ZKVM, Proof>::new(state.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    mock_relayer.expect_start().returning(move |_| {
        let headers_in_box = headers_clone.clone();
        let sender_in_box = sender_clone.clone();

        Box::pin(async move {
            // Send only the first two headers
            for header in headers_in_box.into_iter().take(2) {
                println!("Mocked start sending header: {:?}", header.clone());

                // Simulate sending headers
                sender_in_box
                    .send(header)
                    .expect("Failed to send header in mock");
            }
        })
    });

    let timeout = tokio::spawn(async move {
        println!("Nexus has shut down gracefully.");

        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("Shutting down nexus");
        shutdown_tx.send(true);
    });

    // Spawn the main Nexus logic
    let nexus_task: tokio::task::JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        // The logic should exit or error upon detecting out-of-order headers
        match run_nexus(
            Arc::new(Mutex::new(mock_relayer)),
            node_db_clone.clone(),
            state_machine,
            (prover_mode, 7002),
            state,
            shutdown_rx,
        )
        .await
        {
            Ok(_) => (),
            Err(e) => {
                panic!("Nexus exited with error unexpected error: {:?}", e);
            }
        }

        Ok(())
    });

    let result = tokio::try_join!(nexus_task, timeout).unwrap();

    let mut old_headers: HeaderStore = {
        let db_lock = node_db.lock().await;
        match db_lock.get(b"previous_headers") {
            Ok(Some(i)) => i,
            Ok(None) => panic!("No header store found"),
            Err(_) => {
                panic!("DB Call failed to get previous headers. Restart required.");
            }
        }
    };

    if old_headers.inner().len() != 2 {
        panic!("Should have been two headers.")
    }

    assert_eq!(old_headers.first().unwrap().state_root, H256::zero())
}

#[tokio::test]
async fn test_init_account_tx() {
    use serde_json;
    use tokio::fs;
    let db_path = "./tests/db/test_init_account_tx";
    let app_account_id = AppAccountId::from(AppId(100));

    if let Err(e) = fs::remove_dir_all(db_path.clone()).await {
        eprintln!("Failed to clean up database folder: {:?}", e);
    } else {
        println!("Database folder cleaned up successfully.");
    }

    // Mock the Relayer instance
    let mut mock_relayer = MockRelayer::new();

    // Set up an unbounded channel to simulate sending and receiving headers
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Header>();

    // Mock `receiver` to return the receiver end of the channel
    let receiver_arc: Arc<Mutex<UnboundedReceiver<Header>>> = Arc::new(Mutex::new(receiver));
    let receiver_arc_clone = receiver_arc.clone();

    mock_relayer
        .expect_receiver()
        .returning(move || receiver_arc_clone.clone());

    // Read headers from the JSON file
    let json_path = "tests/data/avail_headers.json";
    let file_content = fs::read_to_string(json_path)
        .await
        .expect("Failed to read headers JSON file");
    let mut headers: Vec<Header> =
        serde_json::from_str(&file_content).expect("Failed to parse headers JSON file");

    let headers_clone = headers.clone();
    let sender_clone = sender.clone();

    let prover_mode = ProverMode::MockProof;
    let (node_db, state) = setup_components(db_path);
    let node_db_clone = node_db.clone();
    let node_db_clone_2 = node_db.clone();
    let state_clone = state.clone();
    let mut state_machine = StateMachine::<ZKVM, Proof>::new(state_clone.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    mock_relayer.expect_start().returning(move |_| {
        let headers_in_box = headers_clone.clone();
        let sender_in_box = sender_clone.clone();
        let node_db_in_box = node_db_clone_2.clone();

        Box::pin(async move {
            let tx_file_path = "tests/data/init_transaction.json";
            sender_in_box
                .send(headers_in_box[0].clone())
                .expect("Failed to send header in mock");
            //TODO: Keep the tests less complicated than below.
            tokio::time::sleep(Duration::from_secs(1)).await;

            let mut old_headers: HeaderStore = {
                let db_lock = node_db_in_box.lock().await;
                match db_lock.get(b"previous_headers") {
                    Ok(Some(i)) => i,
                    Ok(None) => panic!("No header store found"),
                    Err(_) => {
                        panic!("DB Call failed to get previous headers. Restart required.");
                    }
                }
            };

            // Read and deserialize the transaction from the JSON file
            let tx_json = fs::read_to_string(tx_file_path)
                .await
                .expect("Failed to read transaction JSON file");
            let tx: TransactionV2 =
                serde_json::from_str(&tx_json).expect("Failed to parse transaction JSON");

            let response = reqwest::Client::new()
                .post("http://127.0.0.1:7003/tx")
                .json(&tx)
                .send()
                .await
                .unwrap();

            // Check if the request was successful
            if response.status().is_success() {
                ()
            } else {
                panic!(
                    "Post transaction call failed with status code: {}",
                    response.status()
                );
            }

            // Simulate sending headers
            sender_in_box
                .send(headers_in_box[1].clone())
                .expect("Failed to send header in mock");
        })
    });

    let timeout = tokio::spawn(async move {
        println!("Nexus has shut down gracefully.");

        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("Shutting down nexus");
        shutdown_tx.send(true);
    });

    // Spawn the main Nexus logic
    let nexus_task: tokio::task::JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        // The logic should exit or error upon detecting out-of-order headers
        match run_nexus(
            Arc::new(Mutex::new(mock_relayer)),
            node_db_clone.clone(),
            state_machine,
            (prover_mode, 7003),
            state_clone,
            shutdown_rx,
        )
        .await
        {
            Ok(_) => (),
            Err(e) => {
                panic!("Nexus exited with error unexpected error: {:?}", e);
            }
        }

        Ok(())
    });

    tokio::try_join!(nexus_task, timeout).unwrap();

    let old_headers: HeaderStore = {
        let db_lock = node_db.lock().await;
        match db_lock.get(b"previous_headers") {
            Ok(Some(i)) => i,
            Ok(None) => panic!("No header store found"),
            Err(_) => {
                panic!("DB Call failed to get previous headers. Restart required.");
            }
        }
    };

    let state_lock = state.lock().await;
    let current_version = match state_lock.get_version() {
        Ok(Some(i)) => i,
        Ok(None) => panic!("No version found"),
        Err(e) => panic!("Internal db error: {:?}", e),
    };

    let (account_option, _) =
        match state_lock.get_with_proof(&H256::from(app_account_id.0), current_version) {
            Ok(i) => i,
            Err(e) => panic!("State call failed with error: {:?}", e),
        };

    assert_eq!(current_version, 1);
    assert_eq!(
        account_option,
        Some(AccountState {
            height: 0,
            last_proof_height: 0,
            start_nexus_hash: old_headers.inner().last().unwrap().hash().into(),
            state_root: [0u8; 32],
            statement: StatementDigest([1u32; 8]),
        })
    )
}
