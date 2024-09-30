// SPDX-License-Identifier: Apache 2.0
pragma solidity ^0.8.21;

struct Receipt {
    bytes32 chainIdFrom;
    bytes32[] chainIdTo;
    bytes data;
    address from;
    address[] to; // if specified on verification, the callback on "to" function will be called
    uint256 nonce;
}

interface INexusMailbox {
    function receiveMessage(uint256 chainblockNumber, Receipt calldata, bytes calldata proof, bool callback) external;
    function sendMessage(bytes32[] memory chainIdTo, address[] memory to, bytes calldata data) external;
}
