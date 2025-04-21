// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

library Structs {
    struct NexusHeader {
        bytes32 parentHash;
        bytes32 prevStateRoot;
        bytes32 stateRoot;
        bytes32 availHeaderHash;
        uint32 number;
    }

    // Taken from (NexusRollupPI) :
    // https://github.com/availproject/nexus/blob/441dad65a4d617f4d262159aca2b4c6873dd7f5c/core/src/types.rs#L110
    struct Journal {
        bytes32 nexusHash;
        bytes32 stateRoot;
        uint32 height;
        bytes32 startNexusHash;
        bytes32 appId;
        bytes32 imgId;
        bytes32 rollupHash;
    }

    struct AccountState {
        bytes32 statementDigest;
        bytes32 stateRoot;
        bytes32 startNexusHash;
        uint128 lastProofHeight;
        uint128 height;
    }
}
