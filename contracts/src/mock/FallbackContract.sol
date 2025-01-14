// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import {INexusReceiver} from "../interfaces/INexusReceiver.sol";

contract FallbackContract is INexusReceiver {
    event MessageReceived(
        bytes32 indexed fromAppId,
        address indexed from,
        bytes data,
        uint256 nonce
    );

    function onNexusMessage(
        bytes32 fromAppId,
        address from,
        bytes calldata data,
        uint256 nonce
    ) external {
        emit MessageReceived(fromAppId, from, data, nonce);
    }
}
