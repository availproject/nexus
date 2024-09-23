// SPDX-License-Identifier: Apache 2.0
pragma solidity ^0.8.21;

interface INexusReceiver {
    function callback(bytes calldata) external view;
}
