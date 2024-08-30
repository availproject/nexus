## 1. **Overview of Avail Nexus**:
  Avail Nexus addresses the challenge of blockchain fragmentation by providing seamless coordination across different blockchains, allowing assets to remain on their native chains. It enables developers to interact with multiple networks without having to implement state verification individually with each of those networks and users to utilize blockchain services without considering which chain their assets are on.

  To enable secure communication between blockchains, Nexus ensures that each blockchain can independently verify the canonical order and validity of the state transitions on other chains without needing to understand their specific internal mechanics. This is achieved through a verification hub that abstracts domain-specific details.

  While Nexus is technically a rollup, it differs from general-purpose rollups like zkEVMs. Instead, it serves a specific function by allowing accounts to register commitments against ZK programs, enabling them to update their account state by submitting proofs of executed programs.

## 2. **Core Concepts**:
  #### 1. **Nexus**
  - **Role**: Nexus acts as a coordination layer that facilitates cross-chain transactions by aggregating and verifying proofs from different blockchains.
  - **Function**: It allows the creation of accounts by registering verification keys of zk-SNARK programs. These keys define the programs that the accounts will execute. Nexus updates state commitments (like state roots) by verifying proofs that show these programs have run correctly on the previous state.
  - **Execution**: Nexus itself is a zk-Rollup built on the Avail DA layer. It periodically submits aggregated proofs for verification, ensuring the integrity of cross-chain interactions.

  #### 2. **Adapters**
  - **Role**: Adapters are specialized components that allow external rollups to interact with Nexus. They serve as light client (LC) verifiers, running in a zero-knowledge (zk) environment.
  - **Function**: Adapters register verification commitments through Nexus accounts, acting like smart contract verifiers but running on the client side instead of on-chain. This design allows rollups to integrate with Nexus without the complexity of deploying new contracts on each chain.
  - **Example Implementation - ZKSync Adapter**:
    - A specific implementation of an adapter that integrates ZKSync's zk-Rollup with Nexus. It enables ZKSync to interact with Nexus, submit proofs, and update states, ensuring the cross-chain operability of ZKSync within the Nexus ecosystem.

## 3. **Examples and Use Cases**:
   - **ZKSync Example**: Step-by-step guide for setting up and using the ZKSync adapter.
   - **Mock Geth Adapter Example**: Instructions for using the mock adapter for testing purposes.

## 4. **Development Status and Roadmap**:
   - **Current Status**: Overview of what's implemented in Nexus and its adapters.
   - **Component Status**:
     - **Core Nexus**: Current state and pending tasks.
     - **ZKSync Adapter**: Current capabilities and future plans.
     - **Mock Geth Adapter**: Completed features and upcoming improvements.
   - **Future Roadmap**: Planned features and integrations.
