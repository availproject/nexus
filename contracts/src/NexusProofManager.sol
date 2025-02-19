// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import {JellyfishMerkleTreeVerifier} from "./lib/JellyfishMerkleTreeVerifier.sol";
import {RiscZeroVerifierRouter} from "risc0/RiscZeroVerifierRouter.sol";
import {ImageID} from "./GethImageID.sol"; // auto-generated from cargo-build

contract NexusProofManager {
    uint256 public latestNexusBlockNumber = 0;
    RiscZeroVerifierRouter public immutable risc0Router;
    bytes32 public constant imageId = ImageID.ADAPTER_ID; // added for the auto-generated contract
    struct NexusBlock {
        bytes32 stateRoot;
        bytes32 blockHash;
    }

    mapping(uint256 => NexusBlock) public nexusBlock;
    mapping(bytes32 => uint256) public nexusAppIDToLatestBlockNumber;
    mapping(bytes32 => mapping(uint256 => bytes32)) public nexusAppIDToState;

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

    constructor(address _risc0Router) {
        risc0Router = RiscZeroVerifierRouter(_risc0Router);
    }

    // nexus state root
    // updated when we verify the zk proof and then st block updated
    function updateNexusBlock(
        uint256 blockNumber,
        NexusBlock calldata nexusBlockInfo,
        bytes calldata proof,
        bytes calldata journal
    ) external {
        if (nexusBlock[blockNumber].stateRoot != bytes32(0)) {
            revert AlreadyUpdatedBlock(blockNumber);
        }
        nexusBlock[blockNumber] = nexusBlockInfo;
        // TODO: Verify the journal inputs and the updated code.

        // add risc0 verification here
        // ethereum mainnet => 0x8EaB2D97Dfce405A1692a21b3ff3A172d593D319
        // ethereum Holesky => 0xf70aBAb028Eb6F4100A24B203E113D94E87DE93C

        risc0Router.verify(
            proof, // bytes calldata seal
            imageId, // bytes32 ImageID
            sha256(journal) // bytes32 JournalDigest
        );

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
        JellyfishMerkleTreeVerifier.Leaf
        memory leaf = JellyfishMerkleTreeVerifier.Leaf({
            addr: key,
            valueHash: valueHash
        });

        JellyfishMerkleTreeVerifier.Proof
        memory proof = JellyfishMerkleTreeVerifier.Proof({
            leaf: leaf,
            siblings: siblings
        });

        verifyRollupState(nexusBlock[nexusBlockNumber].stateRoot, proof, leaf);

        if (nexusAppIDToLatestBlockNumber[key] < accountState.height) {
            nexusAppIDToLatestBlockNumber[key] = accountState.height;
        }

        nexusAppIDToState[key][accountState.height] = accountState.stateRoot;
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

    function getChainState(
        uint256 blockNumber,
        bytes32 nexusAppID
    ) external view returns (bytes32) {
        uint256 latestBlockNumber = nexusAppIDToLatestBlockNumber[nexusAppID];
        if (blockNumber == 0) {
            return nexusAppIDToState[nexusAppID][latestBlockNumber];
        } else {
            if (blockNumber > latestBlockNumber) {
                revert InvalidBlockNumber(blockNumber, latestBlockNumber);
            }
            return nexusAppIDToState[nexusAppID][blockNumber];
        }
    }
}
