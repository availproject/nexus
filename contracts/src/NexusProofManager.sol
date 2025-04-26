// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import {JellyfishMerkleTreeVerifier} from "./lib/JellyfishMerkleTreeVerifier.sol";
import {RiscZeroVerifierRouter} from "risc0/RiscZeroVerifierRouter.sol";
import {JournalExtractor} from "./lib/JournalExtractor.sol";
import {Structs} from "./lib/Structs.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

contract NexusProofManager is Ownable {
    uint256 public latestNexusBlockNumber = 0;
    RiscZeroVerifierRouter public immutable risc0Router;
    bytes32 public imageId; // added for the auto-generated contract

    mapping(uint256 => Structs.NexusHeader) public nexusHeader;
    mapping(bytes32 => uint256) public nexusAppAddressToLatestBlockNumber;
    mapping(bytes32 => mapping(uint256 => bytes32)) public nexusAppAddressToState;
    mapping(bytes32 => uint256) public availBridgeRootToAvailHeight;

    error AlreadyUpdatedBlock(uint256 blockNumber);
    error InvalidBlockNumber(uint256 blockNumber, uint256 latestBlockNumber);
    error NexusLeafInclusionCheckFailed();
    error InvalidAvailBridgeRootUpdate(uint256 nexusBlockNumber, bytes32 availHeaderHash);

    constructor(address _risc0Router, bytes32 _imageId) Ownable(msg.sender) {
        risc0Router = RiscZeroVerifierRouter(_risc0Router);
        imageId = _imageId;
    }

    // nexus state root
    // updated when we verify the zk proof and then st block updated
    function updateNexusBlock(uint256 blockNumber, bytes calldata _proof, bytes calldata journal) external onlyOwner {
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
    ) external onlyOwner {
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

        if (nexusAppAddressToLatestBlockNumber[key] < accountState.height) {
            nexusAppAddressToLatestBlockNumber[key] = accountState.height;
        }

        nexusAppAddressToState[key][accountState.height] = accountState.stateRoot;
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
    ) external onlyOwner {
        if (nexusHeader[nexusBlockNumber].availHeaderHash != availBlockHash) {
            revert InvalidAvailBridgeRootUpdate(nexusBlockNumber, availBlockHash);
        }
        // TODO : include a verification check after finalization
        // ! Do not use this code in production.
        availBridgeRootToAvailHeight[bridgeRoot] = availBlockNumber;
    }

    function getChainState(uint256 blockNumber, bytes32 nexusAppAddress) external view returns (bytes32) {
        uint256 latestBlockNumber = nexusAppAddressToLatestBlockNumber[nexusAppAddress];
        if (blockNumber == 0) {
            return nexusAppAddressToState[nexusAppAddress][latestBlockNumber];
        } else {
            if (blockNumber > latestBlockNumber) {
                revert InvalidBlockNumber(blockNumber, latestBlockNumber);
            }
            return nexusAppAddressToState[nexusAppAddress][blockNumber];
        }
    }

    function updateImageId(bytes32 _imageId) external onlyOwner {
        imageId = _imageId;
    }
}
