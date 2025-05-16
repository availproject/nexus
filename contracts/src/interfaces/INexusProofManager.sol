// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

//TODO: add variable names instead of just types
interface INexusProofManager {
    function getChainState(uint256, bytes32) external returns (bytes32);
    function nexusAppAddressToLatestBlockNumber(bytes32) external view returns (uint256);
    function availBridgeRootToAvailHeight(bytes32) external view returns (uint256);
    function updateNexusBlock(uint256 blockNumber, bytes calldata _proof, bytes calldata journal) external;
}
