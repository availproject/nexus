// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

library JellyfishMerkleTreeVerifier {
    struct Leaf {
        bytes32 addr;
        bytes32 valueHash;
    }

    struct Proof {
        Leaf leaf;
        bytes32[] siblings;
    }

    function verifyProof(
        bytes32 root,
        Leaf memory leaf,
        Proof memory proof
    ) internal pure returns (bool) {
        if (leaf.addr != 0x0 && leaf.valueHash != 0x0) {
            // Node existence expects inclusion proof
            if (proof.leaf.addr != 0x0 && proof.leaf.valueHash != 0x0) {
                // Prove inclusion with inclusion proof
                if (
                    leaf.addr != proof.leaf.addr ||
                    leaf.valueHash != proof.leaf.valueHash
                ) {
                    return false;
                }
            } else {
                // Expected inclusion proof but get non-inclusion proof passed in
                return false;
            }
        } else {
            // Node absence expects exclusion proof
            if (proof.leaf.addr != 0x0 && proof.leaf.valueHash != 0x0) {
                // The inclusion proof of another node
                if (
                    leaf.addr == proof.leaf.addr ||
                    commonLengthPrefixInBits(leaf.addr, proof.leaf.addr) <
                    proof.siblings.length
                ) {
                    return false;
                }
            }
        }

        bytes32 calculatedRoot;

        if (proof.leaf.addr == 0x0 && proof.leaf.valueHash == 0x0) {
            calculatedRoot = 0x0;
        } else {
            calculatedRoot = sha256(
                abi.encodePacked(
                    hex"4a4d543a3a4c6561664e6f6465",
                    proof.leaf.addr,
                    proof.leaf.valueHash
                )
            );
        }
        uint8[] memory bitValue = calculateBits(
            leaf.addr,
            proof.siblings.length
        );
        for (uint256 i = 0; i < proof.siblings.length; ) {
            if (bitValue[i] == 1) {
                calculatedRoot = sha256(
                    abi.encodePacked(
                        hex"4a4d543a3a496e74726e616c4e6f6465",
                        proof.siblings[i],
                        calculatedRoot
                    )
                );
            } else {
                calculatedRoot = sha256(
                    abi.encodePacked(
                        hex"4a4d543a3a496e74726e616c4e6f6465",
                        calculatedRoot,
                        proof.siblings[i]
                    )
                );
            }
            unchecked {
                ++i;
            }
        }

        return calculatedRoot == root;
    }

    function commonLengthPrefixInBits(
        bytes32 a,
        bytes32 b
    ) private pure returns (uint256) {
        uint256 xor = uint256(a) ^ uint256(b);
        uint256 leadingZeros = 0;
        while (xor > 0) {
            xor >>= 1;
            leadingZeros++;
        }
        return 256 - leadingZeros;
    }

    function calculateBits(
        bytes32 elementKey,
        uint siblingsLen
    ) public pure returns (uint8[] memory) {
        uint skipBits = 256 - siblingsLen;
        uint8[] memory result = new uint8[](siblingsLen);

        uint index = 0;

        // Iterate over the bits in reverse order, starting from 255 down to skipBits (inclusive)
        for (uint i = 255; i >= skipBits && index < siblingsLen; i--) {
            // Extract the bit at position i
            uint bit = (uint(elementKey) >> i) & 1;
            result[index] = uint8(bit);
            index++;
        }

        // Reverse the array to match the expected order
        for (uint i = 0; i < siblingsLen / 2; i++) {
            uint8 temp = result[i];
            result[i] = result[siblingsLen - 1 - i];
            result[siblingsLen - 1 - i] = temp;
        }
        return result;
    }
}
