# To build the adapters and nexus

1. To build/run nexus client :
    Nexus Client is divided into two binaries (to be ran in the specified sequence) :
    - Node
    - Prover

   ```shell
        cd nexus/host
        cargo build --bin "node"/"prover" --no-default-features --features "sp1"/"risc0"

        # To run
        RUST_LOG=info cargo run --bin "node"/"prover" --no-default-features --features "sp1"/"risc0"

        # If running in dev mode for risc 0
        RISC0_DEV_MODE=true RUST_LOG=info cargo run --bin "node"/"prover" --no-default-features --features "risc0" -- --dev

        # If running in mock mode for sp1
        RUST_LOG=info SP1_PROVER=mock cargo run --bin "node"/"prover" --no-default-features --features "sp1" -- --dev

        # If running in cuda mode for sp1
        RUST_LOG=info SP1_PROVER=cuda cargo run --bin "node"/"prover" --no-default-features --features "sp1"
    
        # if running in cuda mode for risc0
        RUST_LOG=info RUSTFLAGS="-C target-cpu=native" cargo run --bin "node"/"prover" --no-default-features --features "risc0-cuda"
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

        # If running in cuda mode for sp1
        RUST_LOG=info SP1_PROVER=cuda cargo run --no-default-features --features "sp1"
        
        # if running in cuda mode for risc0
        RUST_LOG=info RUSTFLAGS="-C target-cpu=native" cargo run --no-default-features --features "risc0-cuda"
   ```

> [!IMPORTANT]
> There can be some changes as there may be different arguments for different adapters