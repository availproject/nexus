# Adapters Overview

Adapters in Avail Nexus are a type of Nexus accounts. These adapters are generally used for updating the state roots of rollups to their respective accounts on Nexus. This design allows the state roots of rollups to be read by anyone verifying Nexus state roots.

Adapters are expected to be zkVM implementations of light client (LC) verifiers for rollups, similar to smart contract verifiers generally deployed on chains like ethereum. Their role is to verify validity proofs, Data Availability (DA) inclusion, and DA ordering, and to generate a proof against the Nexus header up to which they have processed these executions. Nexus headers play a crucial role in ensuring the DA inclusion and DA ordering checks, making them fundamental to the secure operation of adapters within the Nexus ecosystem.

You can find more details about the zksync adapter which serves an example for the upcoming adapter implementations [here](./4_zksync_adapter.md).
