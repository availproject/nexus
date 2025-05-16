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

    struct AppState {
        bytes32 statementDigest;
        bytes32 stateRoot;
        bytes32 startNexusHash;
        uint128 lastProofHeight;
        uint128 height;
    }

    struct AvailBridgeRootVerificationParams {
        bytes32 bridgeRoot;
        uint256 dataRootIndex;
        bytes32 blobRoot;
        bytes32 rangeHash;
        bytes32[] dataRootProof;
    }
}
