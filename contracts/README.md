## Foundry

**Foundry is a blazing fast, portable and modular toolkit for Ethereum application development written in Rust.**

Foundry consists of:

- **Forge**: Ethereum testing framework (like Truffle, Hardhat and DappTools).
- **Cast**: Swiss army knife for interacting with EVM smart contracts, sending transactions and getting chain data.
- **Anvil**: Local Ethereum node, akin to Ganache, Hardhat Network.
- **Chisel**: Fast, utilitarian, and verbose solidity REPL.

## Project Structure

This project contains a variety of Solidity contracts organized into different directories for clarity and modularity.

## src

The `src` directory is the root of all Solidity contracts in the project. It includes the main contracts, interfaces, libraries, mocks, and verification modules.

### Main Contracts

- **NexusProofManager.sol**: The main contract managing Nexus proof mechanisms.

### Interfaces

- **interfaces/INexusProofManager.sol**: Interface for the `NexusProofManager` contract, defining the necessary functions and events.

### Libraries

The `lib` directory contains reusable library contracts, including external libraries and utility functions.

- **lib/JellyfishMerkleTreeVerifier.sol**: Verifies Merkle trees using the Jellyfish algorithm used withing Nexus.

### Verification Modules

The `verification` directory contains contracts related to verification mechanisms, split into Ethereum and zkSync specific implementations.

#### **Ethereum**: `verification/ethereum/Verifier.sol`: General verifier contract for Ethereum.

#### **ZkSync**: `verification/zksync/StorageProof.sol`: Handles storage proof verification for zkSync.

---

## Usage

### Build

```shell
$ forge build
```

### Test

```shell
$ forge test
```

### Format

```shell
$ forge fmt
```

### Gas Snapshots

```shell
$ forge snapshot
```

### Anvil

```shell
$ anvil
```

### Deploy

```shell
$ forge script script/Counter.s.sol:CounterScript --rpc-url <your_rpc_url> --private-key <your_private_key>
```

### Cast

```shell
$ cast <subcommand>
```

### Help

```shell
$ forge --help
$ anvil --help
$ cast --help
```
