use anyhow::Error;
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
        AccountState, AccountWithProof, AppAccountId, AppId, HeaderStore, InitAccount,
        NexusBlockWithTransactions, StatementDigest, SubmitProof, Transaction, TransactionStatus,
        TransactionWithStatus, TxParams, TxSignature, H256,
    },
    zkvm::ProverMode,
};
use nexus_core::{traits::NexusTransaction, types::NexusHeader};
use relayer::Relayer;
use reqwest::Client;
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
    if let Err(e) = fs::remove_dir_all(db_path.clone()).await {
        eprintln!("Failed to clean up database folder: {:?}", e);
    } else {
        println!("Database folder cleaned up successfully.");
    }

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
                println!("Sending header number {}", header.number);
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
            (prover_mode, 6999),
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
    mock_relayer.expect_stop().returning(move || ());

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
        let shutdown_tx_clone = shutdown_tx.clone();

        Box::pin(async move {
            #[cfg(any(feature = "risc0"))]
            let tx_file_path = "tests/data/init_tx_risc0_1.json";

            #[cfg(any(feature = "sp1"))]
            let tx_file_path = "tests/data/init_tx_sp1.json";

            sender_in_box
                .send(headers_in_box[0].clone())
                .expect("Failed to send header in mock");
            //TODO: Keep the tests less complicated than below.
            tokio::time::sleep(Duration::from_secs(1)).await;
            // Read and deserialize the transaction from the JSON file
            let tx_json = fs::read_to_string(tx_file_path)
                .await
                .expect("Failed to read transaction JSON file");
            let tx: Transaction =
                serde_json::from_str(&tx_json).expect("Failed to parse transaction JSON");

            let response = Client::new()
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
            println!("Sent second header");
            // Simulate sending headers
            sender_in_box
                .send(headers_in_box[1].clone())
                .expect("Failed to send header in mock");
            //TODO: Keep the tests less complicated than below.
            tokio::time::sleep(Duration::from_secs(5)).await;
            shutdown_tx_clone.send(true).unwrap();
        })
    });

    // Spawn the main Nexus logic
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
    };

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
    let current_version = match state_lock.get_version(true) {
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
            statement: StatementDigest([
                3963634887, 3768818894, 2608717727, 685163898, 341397292, 1233383743, 1619524616,
                2323598105
            ]),
        })
    )
}

#[cfg(feature = "risc0")]
#[tokio::test]
async fn test_update_tx() {
    use serde_json;
    use tokio::fs;
    let db_path = "./tests/db/test_update_tx";
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
    mock_relayer.expect_stop().returning(move || ());

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
        let shutdown_tx_clone = shutdown_tx.clone();

        Box::pin(async move {
            let init_tx_path = "tests/data/init_tx_risc0_1.json";
            fn submit_proof_tx_path(n: usize) -> String {
                format!("tests/data/submitproof_tx_risc0_{}.json", n)
            }
            for n in 0..10 {
                sender_in_box
                    .send(headers_in_box[n].clone())
                    .expect("Failed to send header in mock");
                //TODO: Keep the tests less complicated than below.
                tokio::time::sleep(Duration::from_secs(1)).await;

                let tx = if n == 0 {
                    // Read and deserialize the transaction from the JSON file
                    let tx_json = fs::read_to_string(init_tx_path)
                        .await
                        .expect("Failed to read transaction JSON file");
                    let tx: Transaction =
                        serde_json::from_str(&tx_json).expect("Failed to parse transaction JSON");

                    tx
                } else {
                    // Read and deserialize the transaction from the JSON file
                    let tx_json = fs::read_to_string(submit_proof_tx_path(n))
                        .await
                        .expect("Failed to read transaction JSON file");
                    let mut tx: Transaction =
                        serde_json::from_str(&tx_json).expect("Failed to parse transaction JSON");

                    tx
                };

                let response = Client::new()
                    .post("http://127.0.0.1:7004/tx")
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
            }

            println!("Sent second header");
            // Simulate sending headers

            sender_in_box
                .send(headers_in_box[10].clone())
                .expect("Failed to send header in mock");

            //TODO: Keep the tests less complicated than below.
            tokio::time::sleep(Duration::from_secs(5)).await;

            shutdown_tx_clone.send(true).unwrap();
        })
    });

    // Spawn the main Nexus logic
    match run_nexus(
        Arc::new(Mutex::new(mock_relayer)),
        node_db_clone.clone(),
        state_machine,
        (prover_mode, 7004),
        state_clone,
        shutdown_rx,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            panic!("Nexus exited with error unexpected error: {:?}", e);
        }
    };

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
    let current_version = match state_lock.get_version(true) {
        Ok(Some(i)) => i,
        Ok(None) => panic!("No version found"),
        Err(e) => panic!("Internal db error: {:?}", e),
    };

    let (account_option, _) =
        match state_lock.get_with_proof(&H256::from(app_account_id.0), current_version) {
            Ok(i) => i,
            Err(e) => panic!("State call failed with error: {:?}", e),
        };

    assert_eq!(current_version, 10);
    assert_eq!(
        account_option,
        Some(AccountState {
            height: 9,
            last_proof_height: 9,
            start_nexus_hash: old_headers.inner().last().unwrap().hash().into(),
            state_root: [
                6, 216, 214, 89, 28, 81, 211, 26, 3, 85, 59, 232, 9, 185, 9, 104, 182, 224, 25,
                245, 45, 166, 90, 253, 133, 157, 64, 140, 201, 186, 98, 21
            ],
            statement: StatementDigest([
                3963634887, 3768818894, 2608717727, 685163898, 341397292, 1233383743, 1619524616,
                2323598105
            ]),
        })
    )
}

#[tokio::test]
async fn test_transaction_status() {
    use serde_json;
    use tokio::fs;
    let db_path = "./tests/db/test_transaction_status";
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
    mock_relayer.expect_stop().returning(move || ());

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
        let shutdown_tx_clone = shutdown_tx.clone();

        Box::pin(async move {
            #[cfg(any(feature = "risc0"))]
            let tx_file_path = "tests/data/init_tx_risc0_1.json";

            #[cfg(any(feature = "sp1"))]
            let tx_file_path = "tests/data/init_tx_sp1.json";

            sender_in_box
                .send(headers_in_box[0].clone())
                .expect("Failed to send header in mock");
            //TODO: Keep the tests less complicated than below.
            tokio::time::sleep(Duration::from_secs(1)).await;

            // Read and deserialize the transaction from the JSON file
            let tx_json = fs::read_to_string(tx_file_path)
                .await
                .expect("Failed to read transaction JSON file");
            let tx: Transaction =
                serde_json::from_str(&tx_json).expect("Failed to parse transaction JSON");

            let response = Client::new()
                .post("http://127.0.0.1:7005/tx")
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
            println!("Sent second header");
            // Simulate sending headers
            sender_in_box
                .send(headers_in_box[1].clone())
                .expect("Failed to send header in mock");
            tokio::time::sleep(Duration::from_secs(2)).await;

            let latest_block: NexusHeader = node_db_in_box
                .lock()
                .await
                .get::<HeaderStore>(b"previous_headers")
                .expect("Unexpected internal db error")
                .expect("Headers must now be committed")
                .first()
                .expect("Headers cannot be empty")
                .clone();
            let response = Client::new()
                .get(format!(
                    "http://127.0.0.1:7005/tx_status?tx_hash={}",
                    hex::encode(tx.hash().as_slice())
                ))
                .send()
                .await
                .unwrap();

            let tx_status: TransactionWithStatus = response.json().await.unwrap();
            println!("\n\nASSEERRTTIINGG");
            assert_eq!(
                tx_status,
                TransactionWithStatus {
                    transaction: tx,
                    status: TransactionStatus::Successful,
                    block_hash: Some(latest_block.hash())
                }
            );

            shutdown_tx_clone.send(true).unwrap();
        })
    });

    // Spawn the main Nexus logic
    match run_nexus(
        Arc::new(Mutex::new(mock_relayer)),
        node_db_clone.clone(),
        state_machine,
        (prover_mode, 7005),
        state_clone,
        shutdown_rx,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            panic!("Nexus exited with unexpected error: {:?}", e);
        }
    };
}

#[tokio::test]
async fn test_get_state_api() {
    use serde_json;
    use tokio::fs;
    let db_path = "./tests/db/test_get_state_api";
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
    mock_relayer.expect_stop().returning(move || ());

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
        let shutdown_tx_clone = shutdown_tx.clone();

        Box::pin(async move {
            fn submit_proof_tx_path(n: usize) -> String {
                format!("tests/data/submitproof_tx_risc0_{}.json", n)
            }
            #[cfg(any(feature = "risc0"))]
            let init_tx_path = "tests/data/init_tx_risc0_1.json";

            #[cfg(any(feature = "sp1"))]
            let init_tx_path = "tests/data/init_tx_sp1.json";

            let mut tx_hashes: Vec<String> = vec![];

            for n in 0..3 {
                sender_in_box
                    .send(headers_in_box[n].clone())
                    .expect("Failed to send header in mock");
                //TODO: Keep the tests less complicated than below.
                tokio::time::sleep(Duration::from_secs(1)).await;

                let tx = if n == 0 {
                    // Read and deserialize the transaction from the JSON file
                    let tx_json = fs::read_to_string(init_tx_path)
                        .await
                        .expect("Failed to read transaction JSON file");
                    let tx: Transaction =
                        serde_json::from_str(&tx_json).expect("Failed to parse transaction JSON");

                    tx
                } else {
                    // Read and deserialize the transaction from the JSON file
                    let tx_json = fs::read_to_string(submit_proof_tx_path(n))
                        .await
                        .expect("Failed to read transaction JSON file");
                    let mut tx: Transaction =
                        serde_json::from_str(&tx_json).expect("Failed to parse transaction JSON");

                    tx
                };

                tx_hashes.push(hex::encode(tx.hash().as_slice()));
                let response = Client::new()
                    .post("http://127.0.0.1:7006/tx")
                    .json(&tx)
                    .send()
                    .await
                    .unwrap();
            }

            // Simulate sending headers
            sender_in_box
                .send(headers_in_box[3].clone())
                .expect("Failed to send header in mock");
            //TODO: Keep the tests less complicated than below.
            tokio::time::sleep(Duration::from_secs(3)).await;

            let response = Client::new()
                .get(format!(
                    "http://127.0.0.1:7006/tx_status?tx_hash={}",
                    tx_hashes[0].clone()
                ))
                .send()
                .await
                .unwrap();

            let tx_status: TransactionWithStatus = response.json().await.unwrap();
            let app_account_id = AppAccountId::from(AppId(100));

            let response = Client::new()
                .get(format!(
                    "http://127.0.0.1:7006/account?app_account_id={}&block_hash={}",
                    hex::encode(app_account_id.0),
                    hex::encode(tx_status.block_hash.unwrap().as_slice())
                ))
                .send()
                .await
                .unwrap();

            let account_with_proof: AccountWithProof =
                response.json().await.expect("API call to nexus failed");
            assert_eq!(
                account_with_proof.account,
                AccountState {
                    height: 0,
                    last_proof_height: 0,
                    start_nexus_hash: [
                        124, 155, 177, 24, 187, 203, 222, 53, 134, 69, 91, 202, 176, 57, 205, 125,
                        6, 190, 127, 189, 221, 197, 246, 121, 254, 142, 231, 94, 10, 210, 115, 246
                    ],
                    state_root: [0u8; 32],
                    statement: StatementDigest([
                        3963634887, 3768818894, 2608717727, 685163898, 341397292, 1233383743,
                        1619524616, 2323598105
                    ]),
                }
            );

            let response = Client::new()
                .get(format!(
                    "http://127.0.0.1:7006/tx_status?tx_hash={}",
                    tx_hashes[1].clone()
                ))
                .send()
                .await
                .unwrap();

            //println!("response: {:?}", response.text().await.unwrap());

            let tx_status: TransactionWithStatus = response.json().await.unwrap();
            let app_account_id = AppAccountId::from(AppId(100));

            if TransactionStatus::Failed == tx_status.status {
                panic!("Transaction should not have failed.")
            }

            let response = Client::new()
                .get(format!(
                    "http://127.0.0.1:7006/account?app_account_id={}&block_hash={}",
                    hex::encode(app_account_id.0),
                    hex::encode(tx_status.block_hash.unwrap().as_slice())
                ))
                .send()
                .await
                .unwrap();

            let account_with_proof: AccountWithProof =
                response.json().await.expect("API call to nexus failed");

            assert_eq!(
                account_with_proof.account,
                AccountState {
                    height: 1,
                    last_proof_height: 1,
                    start_nexus_hash: [
                        124, 155, 177, 24, 187, 203, 222, 53, 134, 69, 91, 202, 176, 57, 205, 125,
                        6, 190, 127, 189, 221, 197, 246, 121, 254, 142, 231, 94, 10, 210, 115, 246
                    ],
                    state_root: [
                        106, 253, 110, 223, 62, 221, 125, 87, 216, 204, 244, 156, 83, 209, 14, 63,
                        0, 95, 50, 234, 154, 7, 99, 193, 144, 166, 121, 106, 221, 81, 90, 138
                    ],
                    statement: StatementDigest([
                        3963634887, 3768818894, 2608717727, 685163898, 341397292, 1233383743,
                        1619524616, 2323598105
                    ]),
                }
            );
            shutdown_tx_clone.send(true).unwrap();
        })
    });

    // Spawn the main Nexus logic
    match run_nexus(
        Arc::new(Mutex::new(mock_relayer)),
        node_db_clone.clone(),
        state_machine,
        (prover_mode, 7006),
        state_clone,
        shutdown_rx,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            panic!("Nexus exited with unexpected error: {:?}", e);
        }
    };
}

#[tokio::test]
async fn test_get_block_api() {
    use serde_json;
    use tokio::fs;
    let db_path = "./tests/db/test_get_block_api";
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
    mock_relayer.expect_stop().returning(move || ());

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
    let state_clone = state.clone();
    let mut state_machine = StateMachine::<ZKVM, Proof>::new(state_clone.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    mock_relayer.expect_start().returning(move |_| {
        let headers_in_box = headers_clone.clone();
        let sender_in_box = sender_clone.clone();
        let shutdown_tx_clone = shutdown_tx.clone();

        Box::pin(async move {
            #[cfg(any(feature = "risc0"))]
            let tx_file_path = "tests/data/init_tx_risc0_1.json";

            #[cfg(any(feature = "sp1"))]
            let tx_file_path = "tests/data/init_tx_sp1.json";

            sender_in_box
                .send(headers_in_box[0].clone())
                .expect("Failed to send header in mock");
            //TODO: Keep the tests less complicated than below.
            tokio::time::sleep(Duration::from_secs(1)).await;

            // Read and deserialize the transaction from the JSON file
            let tx_json = fs::read_to_string(tx_file_path)
                .await
                .expect("Failed to read transaction JSON file");
            let tx: Transaction =
                serde_json::from_str(&tx_json).expect("Failed to parse transaction JSON");

            let response = Client::new()
                .post("http://127.0.0.1:7007/tx")
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
            println!("Sent second header");
            // Simulate sending headers
            sender_in_box
                .send(headers_in_box[1].clone())
                .expect("Failed to send header in mock");
            tokio::time::sleep(Duration::from_secs(2)).await;

            let response = Client::new()
                .get("http://127.0.0.1:7007/block")
                .send()
                .await
                .unwrap();
            let status = response.status();

            let block_with_txs: NexusBlockWithTransactions = response
                .json()
                .await
                .expect("Response with Nexus block not encoded as expected.");

            let json_path = "tests/data/nexus_header_risc0_1.json";
            let file_content = fs::read_to_string(json_path)
                .await
                .expect("Failed to read nexus header json file.");
            let expected_header: NexusBlockWithTransactions =
                serde_json::from_str(&file_content).expect("Failed to parse Nexus header file.");

            assert_eq!(expected_header, block_with_txs);

            shutdown_tx_clone.send(true).unwrap();
        })
    });

    // Spawn the main Nexus logic
    match run_nexus(
        Arc::new(Mutex::new(mock_relayer)),
        node_db_clone.clone(),
        state_machine,
        (prover_mode, 7007),
        state_clone,
        shutdown_rx,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            panic!("Nexus exited with unexpected error: {:?}", e);
        }
    };
}
