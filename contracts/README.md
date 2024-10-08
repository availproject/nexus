## Nexus Contracts

### Project Structure

#### src

The `src` directory is the root of all Solidity contracts in the project. It includes the main contracts, interfaces, libraries, mocks, and verification modules.

#### Main Contracts

- **NexusProofManager.sol**: The main contract managing Nexus proof mechanisms.
- **NexusMailbox.sol**: The mailbox for easy cross-chain messaging.

### Interfaces

- **interfaces/INexusProofManager.sol**: Interface for the `NexusProofManager` contract, defining the necessary functions and events.
- **interfaces/INexusMailbox.sol**: Interface for the `NexusMailbox` contract, defining the necessary functions and events.
- **interfaces/INexusReceiver.sol**: Interface for the receiver contract, defining the necessary functions and events.
- **interfaces/INexusVerifierWrapper.sol**: Interface for the `INexusVerifierWrapper` contract for verification modules, defining the necessary functions and events.

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

### Deploy

- Copy `.env.example` to `.env` and fill the values.
- Copy `deploy-config.example.json` to `deploy-config.json` and fill the values.

```
$ forge script scripts/Nexus.sol --rpc-url <URL> --broadcast
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

---

### ZKSync

#### Instalation

Run the following command to attach zksync to foundry:

```
curl -L https://raw.githubusercontent.com/matter-labs/foundry-zksync/main/install-foundry-zksync | bash
```

Note: ZKSync compiler and deployment process doesn't deploy libraries by default. They have to be deployed seperately and linked via the compiler

#### Build

Find the missing libraries:

```
forge build --zksync --zk-detect-missing-libraries
```

First Deploy libraries:

```
forge create src/lib/JellyfishMerkleTreeVerifier.sol:JellyfishMerkleTreeVerifier --private-key <> --rpc-url <RPC_URL> --chain 271 --zksync
```

Deploy:

```
forge script script/Nexus.sol --rpc-url <RPC_URL> --libraries src/lib/JellyfishMerkleTreeVerifier.sol:JellyfishMerkleTreeVerifier:<ADDRESS_FROM_PREVIOUS_STEP> --zksync
```
