# Overview

The `zksync_adapter` is an example implementation of an adapter within the Avail Nexus ecosystem. It runs as a sidecar to ZKSync's zkStack chains, pulling proofs after proof generation of each batch, and submitting those proofs to Nexus for aggregation. Below is the status of the current development and implementation for the `zksync_adapter`.

## ZKSync Adapter Status

| Task                                        | Status       |
|---------------------------------------------|--------------|
| 🟢 STF                                       | Done         |
| 🟢 Continuity check of proofs                | Done         |
| 🟡 Validity proofs                           | In Progress  |
| 🟡 MPlonk verifier                           | In Progress  |
| 🔴 DA Check                                  | Not Done     |
| 🔴 DA Ordering check                         | Not Done     |

- **Legend:**
  - 🟢 Done
  - 🟡 In Progress
  - 🔴 Not Done

## Setup and Usage

- Instructions and steps for setting up and managing the `zksync_adapter`, can be found in the [Developer guide](./development/2_zksync_example.md).

For more details on Adapters, refer to the [Overview Section](1_overview.md).
