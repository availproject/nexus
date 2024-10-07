// SPDX-License-Identifier: Apache 2.0
pragma solidity ^0.8.21;

struct MailboxMessage {
    bytes32 nexusAppIdFrom;
    bytes32[] nexusAppIdTo;
    bytes data;
    address from;
    address[] to; // if specified on verification, the callback on "to" function will be called
    uint256 nonce;
}

interface INexusMailbox {
    event MailboxEvent(
        bytes32 indexed nexusAppIdFrom,
        bytes32[] nexusAppIdTo,
        bytes data,
        address indexed from,
        address[] to,
        uint256 nonce
    );

    function receiveMessage(
        uint256 chainblockNumber,
        MailboxMessage calldata,
        bytes calldata proof
    ) external;

    function sendMessage(
        bytes32[] memory nexusAppIdFrom,
        address[] memory to,
        uint256 nonce,
        bytes calldata data
    ) external;
}
