// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import {Structs} from "./Structs.sol";
import "forge-std/console.sol";

library JournalExtractor {
    function extractNexusHeader(bytes calldata data) public pure returns (Structs.NexusHeader memory) {
        // parentHash (first 256 bytes, 1 byte from each 4-byte chunk)
        bytes32 parentHash = 0;
        for (uint256 i = 0; i < 32; i++) {
            uint256 bytePos = i * 4;
            if (bytePos < data.length) {
                parentHash = bytes32(uint256(parentHash) << 8 | uint8(data[bytePos]));
            }
        }

        // prevStateRoot (next 256 bytes)
        bytes32 prevStateRoot = 0;
        for (uint256 i = 0; i < 32; i++) {
            uint256 bytePos = 128 + (i * 4);
            if (bytePos < data.length) {
                prevStateRoot = bytes32(uint256(prevStateRoot) << 8 | uint8(data[bytePos]));
            }
        }

        // stateRoot (next 256 bytes)
        bytes32 stateRoot = 0;
        for (uint256 i = 0; i < 32; i++) {
            uint256 bytePos = 256 + (i * 4);
            if (bytePos < data.length) {
                stateRoot = bytes32(uint256(stateRoot) << 8 | uint8(data[bytePos]));
            }
        }

        // availHeaderHash (next 256 bytes)
        bytes32 availHeaderHash = 0;
        for (uint256 i = 0; i < 32; i++) {
            uint256 bytePos = 384 + (i * 4);
            if (bytePos < data.length) {
                availHeaderHash = bytes32(uint256(availHeaderHash) << 8 | uint8(data[bytePos]));
            }
        }

        // number (u32 = 4 bytes)
        uint32 number = 0;
        for (uint256 i = 0; i < 4; i++) {
            uint256 bytePos = 512 + i;
            if (bytePos < data.length) {
                number = number << 8 | uint32(uint8(data[bytePos]));
            }
        }

        return Structs.NexusHeader(parentHash, prevStateRoot, stateRoot, availHeaderHash, number);
    }

    function extractNexusHeaderMockProof(bytes memory data) public pure returns (Structs.NexusHeader memory header) {
        require(data.length == 132, "Invalid data length");

        bytes32 parentHash;
        bytes32 prevStateRoot;
        bytes32 stateRoot;
        bytes32 availHeaderHash;
        uint32 num;

        assembly {
            parentHash := mload(add(data, 32))
            prevStateRoot := mload(add(data, 64))
            stateRoot := mload(add(data, 96))
            availHeaderHash := mload(add(data, 128))
            num := shr(224, mload(add(data, 160))) // only last 4 bytes
        }

        console.log(num);

        header = Structs.NexusHeader(parentHash, prevStateRoot, stateRoot, availHeaderHash, num);
    }
}
