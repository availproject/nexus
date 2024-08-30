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

- Start the Nexus server using the CLI tool:

```zsh
nexus_cli nexus --dev
```

For ZKSync integration, refer to the [ZKSync Example](1_zksync_example.md) guide.
```
