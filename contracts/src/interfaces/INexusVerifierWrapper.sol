// SPDX-License-Identifier: Apache 2.0
pragma solidity ^0.8.21;

interface INexusVerifierWrapper {
    function parseAndVerify(uint256 chainblockNumber, bytes32 receipt, bytes calldata) external;
}
