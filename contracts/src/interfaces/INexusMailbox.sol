// SPDX-License-Identifier: Apache 2.0
pragma solidity ^0.8.21;

struct Receipt {
    bytes32 networkIdFrom;
    bytes32[] networkIdTo;
    bytes data;
    address from;
    address[] to; // if specified on verification, the callback on "to" function will be called
    uint256 nonce;
}

interface INexusMailbox {
    event ReceiptEvent(
        bytes32 indexed networkIdFrom,
        bytes32[] networkIdTo,
        bytes data,
        address indexed from,
        address[] to,
        uint256 nonce
    );

    function receiveMessage(
        uint256 chainblockNumber,
        Receipt calldata,
        bytes calldata proof
    ) external;
    function sendMessage(
        bytes32[] memory networkIdFrom,
        address[] memory to,
        uint256 nonce,
        bytes calldata data
    ) external;
}
