// SPDX-License-Identifier: Apache 2.0
pragma solidity ^0.8.21;

interface INexusReceiver {
    function onNexusMessage(
        bytes32,
        address,
        bytes calldata,
        uint256 nonce
    ) external;
}
