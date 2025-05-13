// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import {JellyfishMerkleTreeVerifier} from "./lib/JellyfishMerkleTreeVerifier.sol";
import {RiscZeroVerifierRouter} from "risc0/RiscZeroVerifierRouter.sol";
import {JournalExtractor} from "./lib/JournalExtractor.sol";
import {Structs} from "./lib/Structs.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {IVectorx} from "./interfaces/IVectorx.sol";
import {Merkle} from "./verification/merkle/Merkle.sol";

contract NexusProofManager is Ownable {
    uint256 public latestNexusBlockNumber = 0;
    RiscZeroVerifierRouter public immutable risc0Router;
    bytes32 public imageId; // added for the auto-generated contract
    IVectorx public vectorX;

    using Merkle for bytes32[];

    mapping(uint256 => Structs.NexusHeader) public nexusHeader;
    mapping(bytes32 => uint256) public nexusAppAddressToLatestBlockNumber;
    mapping(bytes32 => mapping(uint256 => bytes32)) public nexusAppAddressToState;
    mapping(uint256 => bytes32) public availHeightToAvailBridgeRoot;

    error AlreadyUpdatedBlock(uint256 blockNumber);
    error InvalidBlockNumber(uint256 blockNumber, uint256 latestBlockNumber);
    error NexusLeafInclusionCheckFailed();
    error InvalidAvailBridgeRootUpdate(uint256 nexusBlockNumber, bytes32 availHeaderHash);
    error DataRootCommitmentEmpty();
    error InvalidDataRootProof();

    constructor(address _risc0Router, bytes32 _imageId, address _vectorX) Ownable(msg.sender) {
        risc0Router = RiscZeroVerifierRouter(_risc0Router);
        imageId = _imageId;
        vectorX = IVectorx(_vectorX);
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
        bytes32 appAddress,
        Structs.AppState calldata appState
    ) external onlyOwner {
        bytes32 valueHash = sha256(
            abi.encode(
                appState.statementDigest,
                appState.stateRoot,
                appState.startNexusHash,
                appState.lastProofHeight,
                appState.height
            )
        );
        JellyfishMerkleTreeVerifier.Leaf memory leaf =
            JellyfishMerkleTreeVerifier.Leaf({addr: appAddress, valueHash: valueHash});

        JellyfishMerkleTreeVerifier.Proof memory proof =
            JellyfishMerkleTreeVerifier.Proof({leaf: leaf, siblings: siblings});

        verifyRollupState(nexusHeader[nexusBlockNumber].stateRoot, proof, leaf);

        if (nexusAppAddressToLatestBlockNumber[appAddress] < appState.height) {
            nexusAppAddressToLatestBlockNumber[appAddress] = appState.height;
        }

        nexusAppAddressToState[appAddress][appState.height] = appState.stateRoot;
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
        Structs.AvailBridgeRootVerificationParams calldata availBridgeRootVerificationParams
    ) external onlyOwner {
        if (nexusHeader[nexusBlockNumber].availHeaderHash != availBlockHash) {
            revert InvalidAvailBridgeRootUpdate(nexusBlockNumber, availBlockHash);
        }

        bytes32 dataRootCommitment = vectorX.dataRootCommitments(availBridgeRootVerificationParams.rangeHash);
        if (dataRootCommitment == 0x0) {
            revert DataRootCommitmentEmpty();
        }
        if (
            !availBridgeRootVerificationParams.dataRootProof.verifySha2(
                dataRootCommitment,
                availBridgeRootVerificationParams.dataRootIndex,
                keccak256(
                    abi.encode(availBridgeRootVerificationParams.blobRoot, availBridgeRootVerificationParams.bridgeRoot)
                )
            )
        ) {
            revert InvalidDataRootProof();
        }

        availHeightToAvailBridgeRoot[availBlockNumber] = availBridgeRootVerificationParams.bridgeRoot;
    }

    function getChainState(uint256 blockNumber, bytes32 nexusAppAddress) external view returns (bytes32) {
        if (nexusAppAddress == 0x0) {
            return availHeightToAvailBridgeRoot[blockNumber];
        }
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
