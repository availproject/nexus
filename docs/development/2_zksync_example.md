# ZKSync Example with Avail Nexus

This guide will help you set up and run the ZKSync adapter with Avail Nexus.

## Prerequisites

### 1. Clone the ZKSync Repository

- Clone the ZKSync repository:

  ```bash
  git clone https://github.com/vibhurajeev/zksync-era.git
  cd zksync-era
  ```

### 2. Follow the Development Setup
- Complete the development setup.
- Install the zksync CLI for managing zksync services:

  ```bash
  ./bin/zkt
  ```

- Initialize the zkStack ecosystem:

  ```bash
  zksync_inception ecosystem init
  ```

## Running the ZKSync Server

- Start the ZKSync server to run the chain:

  ```bash
  zksync_inception server
  ```

## Running the ZKSync Adapter with Avail Nexus

### 1. Ensure Nexus Server is Running

- Make sure the Nexus server is running. You can specify which ZKVM (`sp1` or `risc0`) to use. If no ZKVM is specified, it will default to `sp1`:

  - To run with the default `sp1` ZKVM:
    ```bash
    nexus_cli nexus --dev
    ```
  - To explicitly run with `sp1`:
    ```bash
    nexus_cli nexus --dev sp1
    ```
  - To run with the `risc0` ZKVM:
    ```bash
    nexus_cli nexus --dev risc0
    ```

### 2. Ensure ZKSync Server is Running

- Make sure the ZKSync server is running as per the previous steps.

### 3. Run the ZKSync Adapter

- Run the ZKSync adapter with the same ZKVM (`sp1` or `risc0`) as the Nexus server. The ZKVM for the adapter must match the one used by the Nexus server:
  - To run with default ports and ZKVM: 
    ```bash
    nexus_cli zksync --dev
    ```
  - To run with the default `sp1` ZKVM:
    ```bash
    nexus_cli zksync --app-id 100 --url http://127.0.0.1:3030 --dev sp1
    ```
  - To run with the `risc0` ZKVM:
    ```bash
    nexus_cli zksync --app-id 100 --url http://127.0.0.1:3030 --dev risc0
    ```
For mock proofs, the `--dev` flag is used; for real proofs, it must be omitted.

## Benchmarking 

- To benchmark change the working directory to `bench` in zksync_adapter
   ```
   cd examples/zksync_adapter/bench
   ```
- To benchmark risc0 implementation of adapter prover run
   ```
   cargo bench
   ```
- To benchmark sp1 implementation of adapter prover run
   ```
   cargo bench --features sp1 --no-default-features 
   ```   
   
### Important Note:
The ZKVM used by the ZKSync adapter must match the ZKVM used by the Nexus server for compatibility.