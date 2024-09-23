// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import {JellyfishMerkleTreeVerifier} from "./lib/JellyfishMerkleTreeVerifier.sol";

contract NexusProofManager {
    uint256 public latestNexusBlockNumber = 0;

    struct NexusBlock {
        bytes32 stateRoot;
        bytes32 blockHash;
    }

    mapping(uint256 => NexusBlock) public nexusBlock;
    mapping(bytes32 => uint256) public nexusAppIdToLatestBlockNumber;
    mapping(bytes32 => mapping(uint256 => bytes32)) public nexusAppIdToState;

    error AlreadyUpdatedBlock(uint256 blockNumber);
    error InvalidBlockNumber(uint256 blockNumber, uint256 latestBlockNumber);
    error NexusLeafInclusionCheckFailed();

    struct AccountState {
        bytes32 statementDigest;
        bytes32 stateRoot;
        bytes32 startNexusHash;
        uint128 lastProofHeight;
        uint128 height;
    }

    // nexus state root
    // updated when we verify the zk proof and then st block updated
    function updateNexusBlock(uint256 blockNumber, NexusBlock calldata nexusBlockInfo) external {
        if (nexusBlock[blockNumber].stateRoot != bytes32(0)) {
            revert AlreadyUpdatedBlock(blockNumber);
        }
        nexusBlock[blockNumber] = nexusBlockInfo;
        // TODO: verify a zk proof from nexus

        if (blockNumber > latestNexusBlockNumber) {
            latestNexusBlockNumber = blockNumber;
        }
    }

    function updateChainState(
        uint256 nexusBlockNumber,
        bytes32[] calldata siblings,
        bytes32 key,
        AccountState calldata accountState
    ) external {
        bytes32 valueHash = sha256(
            abi.encode(
                accountState.statementDigest,
                accountState.stateRoot,
                accountState.startNexusHash,
                accountState.lastProofHeight,
                accountState.height
            )
        );
        JellyfishMerkleTreeVerifier.Leaf memory leaf =
            JellyfishMerkleTreeVerifier.Leaf({addr: key, valueHash: valueHash});

        JellyfishMerkleTreeVerifier.Proof memory proof =
            JellyfishMerkleTreeVerifier.Proof({leaf: leaf, siblings: siblings});

        verifyRollupState(nexusBlock[nexusBlockNumber].stateRoot, proof, leaf);

        if (nexusAppIdToLatestBlockNumber[key] < accountState.height) {
            nexusAppIdToLatestBlockNumber[key] = accountState.height;
        }

        nexusAppIdToState[key][accountState.height] = accountState.stateRoot;
    }

    function verifyRollupState(
        bytes32 root,
        JellyfishMerkleTreeVerifier.Proof memory proof,
        JellyfishMerkleTreeVerifier.Leaf memory leaf
    ) public pure {
        if (!JellyfishMerkleTreeVerifier.verifyProof(root, leaf, proof)) {
            revert NexusLeafInclusionCheckFailed();
        }
    }

    function getChainState(uint256 blockNumber, bytes32 nexusAppID) external view returns (bytes32) {
        uint256 latestBlockNumber = nexusAppIdToLatestBlockNumber[nexusAppID];
        if (blockNumber == 0) {
            return nexusAppIdToState[nexusAppID][latestBlockNumber];
        } else {
            if (blockNumber > latestBlockNumber) {
                revert InvalidBlockNumber(blockNumber, latestBlockNumber);
            }
            return nexusAppIdToState[nexusAppID][blockNumber];
        }
    }
}
