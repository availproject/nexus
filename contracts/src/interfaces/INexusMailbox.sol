// SPDX-License-Identifier: Apache 2.0
pragma solidity ^0.8.21;

struct MailboxMessage {
    bytes32 nexusAppIDFrom;
    bytes32[] nexusAppIDTo;
    bytes data;
    address from;
    address[] to; // if specified on verification, the callback on "to" function will be called
    uint256 nonce; // TODO: Check if nonce can be moved to data field to be handled by sender.
}

interface INexusMailbox {
    event MailboxEvent(
        bytes32 indexed nexusAppIDFrom,
        bytes32[] nexusAppIDTo,
        bytes data,
        address indexed from,
        address[] to,
        uint256 nonce,
        bytes32 receiptHash
    );

    function receiveMessage(
        uint256 chainblockNumber,
        MailboxMessage calldata,
        bytes calldata proof
    ) external;

    function sendMessage(
        bytes32[] memory nexusAppIdTo,
        address[] memory to,
        uint256 nonce,
        bytes calldata data
    ) external;
}
