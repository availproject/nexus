# Examples

The `examples/` directory contains practical implementations that demonstrate how to interact with Avail Nexus. These examples cover different use cases and adapters, helping you understand how to integrate and utilize Nexus within various blockchain environments.

### 1. **demo_rollup**

The `demo_rollup` example illustrates how a non-EVM rollup can directly interact with Nexus. It serves as a basic guide for developers looking to integrate custom rollups with Nexus.

### 2. **mock_geth_adapter**

The `mock_geth_adapter` is a mock implementation designed for testing purposes. It updates the state roots of any Geth instances onto Nexus, simulating the adapter flow without requiring a full zk-rollup. This example is ideal for developers who want to experiment with their adapter logic.

### 3. **zksync_adapter**

The `zksync_adapter` example demonstrates how to integrate Nexus with ZKSync's zkStack chains. This adapter runs as a sidecar to the ZKSync network, pulling proofs generated for each batch and submitting them to Nexus for aggregation. It's a powerful example for developers working with zkSync or similar zk-rollup solutions, offering a clear pathway to leverage Nexus for proof aggregation and cross-chain interoperability.
