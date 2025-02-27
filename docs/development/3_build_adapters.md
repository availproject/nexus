# To build the adapters and nexus

1. To build/run nexus client :

    ```shell
    cd nexus/host
    cargo build --no-default-features --features "sp1"/"risc0"

   # To run
    RUST_LOG=info cargo run --no-default-features --features "sp1"/"risc0"

   # If running in dev mode for risc 0
   RISC0_DEV_MODE=true RUST_LOG=info cargo run --no-default-features --features "risc0" -- --dev
   
   # If running in mock mode for sp1
   RUST_LOG=info SP1_PROVER=mock cargo run --no-default-features --features "sp1" -- --dev
    ```

2. To build adapters :

   ```shell
   cd examples/<adapter_name>
   cargo build --no-default-features --features "sp1"/"risc0"

   # To run
    RUST_LOG=info cargo run --no-default-features --features "sp1"/"risc0"

   # If running in dev mode for risc 0
   RISC0_DEV_MODE=true RUST_LOG=info cargo run --no-default-features --features "risc0" -- --dev
   
   # If running in mock mode for sp1
   RUST_LOG=info SP1_PROVER=mock cargo run --no-default-features --features "sp1" -- --dev
   ```
