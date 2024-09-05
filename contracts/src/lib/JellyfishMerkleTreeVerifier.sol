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

    function verifyProof(bytes32 root, Leaf calldata leaf, Proof calldata proof) external pure returns (bool) {
        if (leaf.addr != 0x0 && leaf.valueHash != 0x0) {
            // Node existence expects inclusion proof
            if (proof.leaf.addr != 0x0 && proof.leaf.valueHash != 0x0) {
                // Prove inclusion with inclusion proof
                if (leaf.addr != proof.leaf.addr || leaf.valueHash != proof.leaf.valueHash) {
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
                    leaf.addr == proof.leaf.addr
                        || commonLengthPrefixInBits(leaf.addr, proof.leaf.addr) < proof.siblings.length
                ) {
                    return false;
                }
            }
        }

        bytes32 calculatedRoot;

        if (proof.leaf.addr == 0x0 && proof.leaf.valueHash == 0x0) {
            calculatedRoot = 0x0;
        } else {
            calculatedRoot =
                sha256(abi.encodePacked(hex"4a4d543a3a4c6561664e6f6465", proof.leaf.addr, proof.leaf.valueHash));
        }
        for (uint256 i = 0; i < proof.siblings.length;) {
            if (getIthBit(leaf.addr, proof.siblings.length - i) == 1) {
                calculatedRoot =
                    sha256(abi.encodePacked(hex"4a4d543a3a496e74726e616c4e6f6465", proof.siblings[i], calculatedRoot));
            } else {
                calculatedRoot =
                    sha256(abi.encodePacked(hex"4a4d543a3a496e74726e616c4e6f6465", calculatedRoot, proof.siblings[i]));
            }
            unchecked {
                ++i;
            }
        }

        return calculatedRoot == root;
    }

    function commonLengthPrefixInBits(bytes32 a, bytes32 b) private pure returns (uint256) {
        uint256 xor = uint256(a) ^ uint256(b);
        uint256 leadingZeros = 0;
        while (xor > 0) {
            xor >>= 1;
            leadingZeros++;
        }
        return 256 - leadingZeros;
    }

    function getIthBit(bytes32 value, uint256 i) private pure returns (uint256) {
        return (uint256(value) >> i) & 1;
    }
}
