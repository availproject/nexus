// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

library JournalExtractor {

    // Taken from (NexusRollupPI) :
    // https://github.com/availproject/nexus/blob/441dad65a4d617f4d262159aca2b4c6873dd7f5c/core/src/types.rs#L110
    struct Journal {
        bytes32 nexusHash;
        bytes32 stateRoot;
        uint32 height;
        bytes32 startNexusHash;
        bytes32 appId;
        bytes32 imgId;
        bytes32 rollupHash;
    }

    function extractJournal(bytes calldata journal) public pure returns(Journal memory) {
        // For nexusHash (first 256 bytes, extracting first byte from each 4-byte chunk)
        bytes32 nexusHash = 0;
        for (uint256 i = 0; i < 32; i++) {
            uint256 bytePos = i * 4; // Position in the journal
            if (bytePos < journal.length) {
                // Shift left by 8 bits (1 byte) and add the new byte
                nexusHash = bytes32(uint256(nexusHash) << 8 | uint8(journal[bytePos]));
            }
        }

        // For stateRoot (next 256 bytes)
        bytes32 stateRoot = 0;
        for (uint256 i = 0; i < 32; i++) {
            uint256 bytePos = 128 + (i * 4);
            if (bytePos < journal.length) {
                stateRoot = bytes32(uint256(stateRoot) << 8 | uint8(journal[bytePos]));
            }
        }

        // For height (next 4 bytes)
        uint32 height = 0;
        for (uint256 i = 0; i < 4; i++) {
            uint256 bytePos = 129 + i;
            if (bytePos < journal.length) {
                height = height << 8 | uint32(uint8(journal[bytePos]));
            }
        }

        // For startNexusHash (next 256 bytes)
        bytes32 startNexusHash = 0;
        for (uint256 i = 0; i < 32; i++) {
            uint256 bytePos = 260 + (i * 4);
            if (bytePos < journal.length) {
                startNexusHash = bytes32(uint256(startNexusHash) << 8 | uint8(journal[bytePos]));
            }
        }

        // For appId (next 256 bytes)
        bytes32 appId = 0;
        for (uint256 i = 0; i < 32; i++) {
            uint256 bytePos = 388 + (i * 4);
            if (bytePos < journal.length) {
                appId = bytes32(uint256(appId) << 8 | uint8(journal[bytePos]));
            }
        }

        // For imgId (direct bytes32)
        bytes32 imgId;
        uint256 imgOffset = 516;
        if (imgOffset + 32 <= journal.length) {
            assembly {
                imgId := calldataload(add(journal.offset, imgOffset))
            }
        }

        // For rollupHash (next 256 bytes)
        bytes32 rollupHash = 0;
        for (uint256 i = 0; i < 32; i++) {
            uint256 bytePos = 550 + (i * 4);
            if (bytePos < journal.length) {
                rollupHash = bytes32(uint256(rollupHash) << 8 | uint8(journal[bytePos]));
            }
        }

        return Journal(nexusHash, stateRoot, height, startNexusHash, appId, imgId, rollupHash);
    }
}
