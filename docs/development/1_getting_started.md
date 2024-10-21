# Getting Started with Avail Nexus

This guide will help you set up Avail Nexus and get started with running the Nexus server.

## Prerequisites

### 1. Install Rust

- Install Rust by following the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).

### 2. Install RISC Zero Toolchain

- Install the RISC Zero toolchain installer
```zsh
curl -L https://risczero.com/install | bash
```
- Install the RISC Zero toolchain 
```zsh 
rzup
```
Running rzup will install the latest version of the RISC Zero toolchain.

Read more about RISC Zero installation in their guide [here](https://dev.risczero.com/api/zkvm/install).

## Setting Up the Environment

### 1. Run the Setup Script

- Execute the `setup.sh` script to configure your environment. _(Note: This script is tailored for Zsh.)_

```zsh
./setup.sh
```

### 2. Restart the CLI Session

- Restart your terminal or CLI session to ensure that all environment variables are correctly loaded.

## Running the Nexus Server

The Nexus server can be run with different Zero-Knowledge Virtual Machines (ZKVM). You can specify which ZKVM to use by adding the appropriate flag (`sp1` or `risc0`). If no ZKVM is specified, it will default to **sp1**.

### Command Options:
- To run the Nexus server using the default ZKVM (`sp1`):
  ```bash
  nexus_cli nexus --dev
  ```
- To run the Nexus server with the `sp1` ZKVM explicitly:
  ```bash
  nexus_cli nexus --dev sp1
  ```
- To run the Nexus server with the `risc0` ZKVM:
  ```bash
  nexus_cli nexus --dev risc0
  ```

For mock proofs, the `--dev` flag is used; for real proofs, it must be omitted.

### Important Note:
Make sure that any example adapters you are running are also configured to use the **same ZKVM** as the one chosen for the Nexus server. The ZKVM for the adapters and the server must match in order for them to work correctly.

For ZKSync integration, refer to the [ZKSync Example](2_zksync_example.md) guide.
