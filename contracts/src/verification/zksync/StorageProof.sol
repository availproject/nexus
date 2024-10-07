// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import {SparseMerkleTree, TreeEntry} from "./SparseMerkleTree.sol";

/// @notice Interface for the zkSync's contract
interface IZKSyncNexusManagerRouter {
    /// @notice Returns the hash of the stored batch
    function storedBatchHash(uint256) external view returns (bytes32);
}

/// @notice Storage proof that proves a storage key-value pair is included in the batch
struct StorageProof {
    uint64 batchNumber;
    // Account and key-value pair of its storage
    address account;
    uint256 key;
    bytes32 value;
    // Proof path and leaf index
    bytes32[] path;
    uint64 index;
}

contract StorageProofVerifier {
    IZKSyncNexusManagerRouter public immutable zksyncDiamondAddress;
    SparseMerkleTree public smt;

    constructor(
        IZKSyncNexusManagerRouter _zksyncDiamondAddress,
        SparseMerkleTree _smt
    ) {
        zksyncDiamondAddress = _zksyncDiamondAddress;
        smt = _smt;
    }

    /// @notice Verifies the storage proof
    function verify(
        StorageProof memory _proof
    ) public view returns (bool valid) {
        // Fold the proof path to get hash of L2 state
        bytes32 l2BatchHash = smt.getRootHash(
            _proof.path,
            TreeEntry({
                key: _proof.key,
                value: _proof.value,
                leafIndex: _proof.index
            }),
            _proof.account
        );

        bytes32 l1BatchHash = zksyncDiamondAddress.storedBatchHash(
            _proof.batchNumber
        );

        valid = l2BatchHash == l1BatchHash;
    }
}
