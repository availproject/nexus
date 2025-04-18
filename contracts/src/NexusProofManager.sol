// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import {JellyfishMerkleTreeVerifier} from "./lib/JellyfishMerkleTreeVerifier.sol";
import {RiscZeroVerifierRouter} from "risc0/RiscZeroVerifierRouter.sol";
import {ImageID} from "./GethImageID.sol"; // auto-generated from cargo-build
import {JournalExtractor} from "./lib/JournalExtractor.sol";
import {Structs} from "./lib/Structs.sol";

contract NexusProofManager {
    uint256 public latestNexusBlockNumber = 0;
    RiscZeroVerifierRouter public immutable risc0Router;
    bytes32 public constant imageId = ImageID.ADAPTER_ID; // added for the auto-generated contract

    mapping(uint256 => Structs.NexusHeader) public nexusHeader;
    mapping(bytes32 => uint256) public nexusAppIDToLatestBlockNumber;
    mapping(bytes32 => mapping(uint256 => bytes32)) public nexusAppIDToState;
    mapping(bytes32 => uint256) public availBridgeRootToAvailHeight;

    error AlreadyUpdatedBlock(uint256 blockNumber);
    error InvalidBlockNumber(uint256 blockNumber, uint256 latestBlockNumber);
    error NexusLeafInclusionCheckFailed();
    error InvalidAvailBridgeRootUpdate(uint256 nexusBlockNumber, bytes32 availHeaderHash);

    constructor(address _risc0Router) {
        risc0Router = RiscZeroVerifierRouter(_risc0Router);
    }

    // nexus state root
    // updated when we verify the zk proof and then st block updated
    function updateNexusBlock(uint256 blockNumber, bytes calldata proof, bytes calldata journal) external {
        if (nexusHeader[blockNumber].stateRoot != bytes32(0)) {
            revert AlreadyUpdatedBlock(blockNumber);
        }

        Structs.NexusHeader memory nexusHeaderStruct = JournalExtractor.extractNexusHeader(journal);

        nexusHeader[blockNumber] = Structs.NexusHeader({
            parentHash: nexusHeaderStruct.parentHash, 
            prevStateRoot: nexusHeaderStruct.prevStateRoot,
            stateRoot: nexusHeaderStruct.stateRoot,
            availHeaderHash: nexusHeaderStruct.availHeaderHash,
            number: nexusHeaderStruct.number
        });

        // To be uncommented after proving PR is merged.
        // risc0Router.verify(
        //     proof, // bytes calldata seal
        //     imageId, // bytes32 ImageID
        //     sha256(journal) // bytes32 JournalDigest
        // );

        if (blockNumber > latestNexusBlockNumber) {
            latestNexusBlockNumber = blockNumber;
        }
    }

    function updateChainState(
        uint256 nexusBlockNumber,
        bytes32[] calldata siblings,
        bytes32 key,
        Structs.AccountState calldata accountState
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

        verifyRollupState(nexusHeader[nexusBlockNumber].stateRoot, proof, leaf);

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

    function updateAvailBridgeRoot(
        uint256 nexusBlockNumber,
        bytes32 availBlockHash,
        uint256 availBlockNumber,
        bytes32 bridgeRoot
    ) external {
        require(nexusHeader[nexusBlockNumber].availHeaderHash == availBlockHash) {
            revert InvalidAvailBridgeRootUpdate(nexusBlockNumber, availBlockHash);
        }
        // TODO : include a verification check after finalization
        // ! Do not use this code in production.
        availBridgeRootToAvailHeight[bridgeRoot] = availBlockNumber;
    }

    function getChainState(uint256 blockNumber, bytes32 nexusAppID) external view returns (bytes32) {
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
