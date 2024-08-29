# Avail Nexus

**Scaling blockchains to everyday users** will be heavily reliant upon rollups and multiple chains. While this will enable vastly expanding capabilities for blockchains, it will also increase fragmentation among users, developers, and functionality spread across different blockchains and ecosystems. We already live in a multi-chain world, and providing seamless coordination between them would have immediate benefits for the whole ecosystem today and into the future.

**Avail Nexus** offers seamless usability across different blockchains and ecosystems without users having to think about which chain their assets are on or developers needing to manage connections with multiple networks. Avail Nexus achieves this through proof aggregation and using Avail DA (Data Availability) as its root of trust. By aggregating proofs from different ecosystems and harnessing Avail DA’s ability to quickly verify data availability, Nexus facilitates cross-chain transactions in a trust-minimized, permissionless, and seamless way. 

### Repository Structure

- **core/**: Contains the foundational package that serves as the backbone for Nexus, its adapters, and any participants interacting with Nexus. This directory houses all the shared logic and common code.

- **nexus/**: Encompasses the core logic of the Nexus server, responsible for running the rollup and generating aggregate proofs.

- **examples/**: Provides practical examples of various adapter implementations that can be utilized with Nexus.

- **nexus-cli/**: A straightforward command-line tool designed to manage all Nexus services efficiently.

### Getting Started

Follow these steps to set up Avail Nexus and get started with running the Nexus server and submitting proofs.

#### 1. Install Prerequisites

- **Install Rust**: If you haven't already installed Rust, you can do so by following the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).

- **Install RISC Zero Tool**: Avail Nexus uses the RISC Zero toolchain for zero-knowledge proofs. Install it by following the guide [here](https://dev.risczero.com/api/zkvm/install).

#### 2. Run the Setup Script

- Execute the `setup.sh` script to configure your environment. _(Note: This script is currently tailored for Zsh.)_

```zsh
./setup.sh
```

#### 3. Restart the CLI

- After running the setup script, restart your terminal or CLI session to apply the changes and ensure that all paths and environment variables are correctly loaded.

#### 4. Run the Nexus Server

- Use the CLI tool to start the Nexus server. You can add the `--dev` flag to run with mock proofs for development purposes.

```zsh
nexus_cli nexus --dev
```

#### 5. Explore and Run Examples

- To submit proofs and interact with Nexus, refer to the examples provided in the `examples/` directory. These examples demonstrate how to use various adapters with Nexus.
