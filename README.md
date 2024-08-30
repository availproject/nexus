# Avail Nexus

**Scaling blockchains to everyday users** will be heavily reliant upon rollups and multiple chains. While this will enable vastly expanding capabilities for blockchains, it will also increase fragmentation among users, developers, and functionality spread across different blockchains and ecosystems. We already live in a multi-chain world, and providing seamless coordination between them would have immediate benefits for the whole ecosystem today and into the future.

**Avail Nexus** offers seamless usability across different blockchains and ecosystems without users having to think about which chain their assets are on or developers needing to manage connections with multiple networks. Avail Nexus achieves this through proof aggregation and using Avail DA (Data Availability) as its root of trust. By aggregating proofs from different ecosystems and harnessing Avail DA’s ability to quickly verify data availability, Nexus facilitates cross-chain transactions in a trust-minimized, permissionless, and seamless way.

To read more about nexus and its architecture refer to the [Overview](./docs/1_overview.md).

### Repository Structure

- **core/**: Contains the foundational package that serves as the backbone for Nexus, its adapters, and any participants interacting with Nexus.
- **nexus/**: Encompasses the core logic of the Nexus server, responsible for running the rollup and generating aggregate proofs.
- **examples/**: Provides practical examples of various adapter implementations that can be utilized with Nexus.
- **nexus-cli/**: A command-line tool designed to manage all Nexus services efficiently.

### Getting Started

To set up Avail Nexus, follow the instructions in the [Getting Started Guide](./docs/development/1_getting_started.md).

For detailed usage of the ZKSync integration, refer to the [ZKSync Example](./docs/development/2_zksync_example.md).
