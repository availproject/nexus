// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

//TODO: add variable names instead of just types
interface INexusProofManager {
    function getStorageRoot(bytes32, uint256, address, bytes calldata) external view returns (bytes32);
    function getChainState(uint256, bytes32) external returns (bytes32);
    function nexusAppIDToLatestBlockNumber(bytes32) external view returns (uint256);
    function nexusAppIDToState(bytes32, uint256) external view returns (bytes32);
}
