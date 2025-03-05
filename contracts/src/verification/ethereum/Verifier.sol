// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import "./StorageProof.sol";
import "../../interfaces/INexusProofManager.sol";

contract EthereumVerifier is StorageProof {
    INexusProofManager immutable nexusStateManager;

    constructor(INexusProofManager _nexusStateManager) {
        nexusStateManager = _nexusStateManager;
    }

    function getStorageRoot(
        bytes32 nexusAppID,
        uint256 chainBlockNumber,
        address account,
        bytes calldata accountTrieProof
    ) external view returns (bytes32) {
        require(chainBlockNumber <= nexusStateManager.nexusAppIDToLatestBlockNumber(nexusAppID), "Invalid block number");
        bytes32 stateRoot = nexusStateManager.nexusAppIDToState(nexusAppID, chainBlockNumber);
        (,,, bytes32 storageRoot) = verifyAccount(stateRoot, accountTrieProof, account);
        return storageRoot;
    }
}
