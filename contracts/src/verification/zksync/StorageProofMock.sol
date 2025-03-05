// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.10;

import "./StorageProof.sol";

contract StorageProofMock {
    IZKSyncNexusManagerRouter public immutable zksyncDiamondAddress;
    SparseMerkleTree public smt;

    constructor(IZKSyncNexusManagerRouter _zksyncDiamondAddress, SparseMerkleTree _smt) {
        zksyncDiamondAddress = _zksyncDiamondAddress;
        smt = _smt;
    }

    function verify(StorageProof memory _proof, uint256 _key) public view returns (bool valid) {
        valid = true;
    }
}
