// SPDX-License-Identifier: Apache-2.0
// Modified from https://github.com/QEDK/solidity-misc/blob/master/contracts/Merkle.sol
pragma solidity ^0.8.21;

/**
 * @author  @QEDK
 * @title   Merkle
 * @notice  A gas-efficient ordered Merkle proof of inclusion implementation
 * @custom:security security@availproject.org
 */
library Merkle {
    function verify(bytes32[] calldata proof, bytes32 root, uint256 index, bytes32 leaf)
        internal
        pure
        returns (bool isValid)
    {
        assembly ("memory-safe") {
            if proof.length {
                // set end to be the end of the proof array, shl(5, proof.length) is equivalent to proof.length * 32
                let end := add(proof.offset, shl(5, proof.length))
                // set iterator to the start of the proof array
                let i := proof.offset
                // prettier-ignore
                // solhint-disable-next-line no-empty-blocks
                for {} 1 {} {
                    // if index is odd, leaf slot is at 0x20, else 0x0
                    let leafSlot := shl(5, and(0x1, index))
                    // store the leaf at the calculated slot
                    mstore(leafSlot, leaf)
                    // store proof element in whichever slot is not occupied by the leaf
                    mstore(xor(leafSlot, 32), calldataload(i))
                    // hash the first 64 bytes in memory
                    leaf := keccak256(0, 64)
                    // shift index right by 1 bit to divide by 2
                    index := shr(1, index)
                    // increment iterator by 32 bytes
                    i := add(i, 32)
                    // break if iterator is at the end of the proof array
                    if iszero(lt(i, end)) { break }
                }
            }
            // check if index is zeroed out (because tree is balanced) and leaf is equal to root
            isValid := and(eq(leaf, root), iszero(index))
        }
    }

    function verifySha2(bytes32[] calldata proof, bytes32 root, uint256 index, bytes32 leaf)
        internal
        view
        returns (bool isValid)
    {
        assembly ("memory-safe") {
            if proof.length {
                // set end to be the end of the proof array, shl(5, proof.length) is equivalent to proof.length * 32
                let end := add(proof.offset, shl(5, proof.length))
                // set iterator to the start of the proof array
                let i := proof.offset
                // prettier-ignore
                // solhint-disable-next-line no-empty-blocks
                for {} 1 {} {
                    // if index is odd, leaf slot is at 0x20, else 0x0
                    let leafSlot := shl(5, and(0x1, index))
                    // store the leaf at the calculated slot
                    mstore(leafSlot, leaf)
                    // store proof element in whichever slot is not occupied by the leaf
                    mstore(xor(leafSlot, 32), calldataload(i))
                    // hash the first 64 bytes in memory with sha2-256 and store in scratch space
                    if iszero(staticcall(gas(), 0x02, 0, 64, 0, 32)) { break }
                    // store for next iteration
                    leaf := mload(0)
                    // shift index right by 1 bit to divide by 2
                    index := shr(1, index)
                    // increment iterator by 32 bytes
                    i := add(i, 32)
                    // break if iterator is at the end of the proof array
                    if iszero(lt(i, end)) { break }
                }
            }
            // check if index is zeroed out (because tree is balanced) and leaf is equal to root
            isValid := and(eq(leaf, root), iszero(index))
        }
    }

    function verifySha2Memory(bytes32[] memory proof, bytes32 root, uint256 index, bytes32 leaf)
        internal
        view
        returns (bool isValid)
    {
        assembly ("memory-safe") {
            if mload(proof) {
                // initialize iterator to the offset of proof elements in memory
                let i := add(proof, 32)
                // left shift by 5 is equivalent to multiplying by 32
                let end := add(i, shl(5, mload(proof)))
                // prettier-ignore
                // solhint-disable-next-line no-empty-blocks
                for {} 1 {} {
                    // if index is odd, leaf slot is at 0x20, else 0x0
                    let leafSlot := shl(5, and(0x1, index))
                    // store the leaf at the calculated slot
                    mstore(leafSlot, leaf)
                    // store proof element in whichever slot is not occupied by the leaf
                    mstore(xor(leafSlot, 32), mload(i))
                    // hash the first 64 bytes in memory with sha2-256 and store in scratch space
                    if iszero(staticcall(gas(), 0x02, 0, 64, 0, 32)) { break }
                    // store for next iteration
                    leaf := mload(0)
                    // shift index right by 1 bit to divide by 2
                    index := shr(1, index)
                    // increment iterator by 32 bytes
                    i := add(i, 32)
                    // break if iterator is at the end of the proof array
                    if iszero(lt(i, end)) { break }
                }
            }
            // check if index is zeroed out (because tree is balanced) and leaf is equal to root
            isValid := and(eq(leaf, root), iszero(index))
        }
    }
}
