// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import {StorageProof, StorageProofVerifier, IZKSyncNexusManagerRouter} from "./StorageProof.sol";
import {SparseMerkleTree} from "./SparseMerkleTree.sol";
import {INexusVerifierWrapper} from "../../interfaces/INexusVerifierWrapper.sol";
import {INexusProofManager} from "../../interfaces/INexusProofManager.sol";

contract VerifierWrapper is INexusVerifierWrapper, StorageProofVerifier {
    error InvalidProof();
    error VerificationFailed();

    constructor(
        IZKSyncNexusManagerRouter zksyncDiamondAddress,
        SparseMerkleTree smt
    ) StorageProofVerifier(zksyncDiamondAddress, smt) {}

    function parseAndVerify(
        uint256,
        bytes32 receipt,
        bytes calldata data
    ) external view {
        StorageProof memory proof = abi.decode(data, (StorageProof));
        if (proof.value == bytes32(0)) {
            revert InvalidProof();
        }

        if (
            verify(
                proof,
                uint256(
                    keccak256(abi.encodePacked(uint256(0), uint256(receipt)))
                )
            )
        ) {
            revert VerificationFailed();
        }
    }
}
