// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import "../../interfaces/INexusProofManager.sol";

contract ZKSyncNexusManagerRouter {
    INexusProofManager immutable proofManager;
    bytes32 immutable chainId;
    uint256 defaultBlockNumber = 0;

    constructor(INexusProofManager _proofManager, bytes32 _chainId) {
        proofManager = _proofManager;
        chainId = _chainId;
    }

    function storedBatchHash(uint256) external returns (bytes32) {
        return proofManager.getChainState(defaultBlockNumber, chainId);
    }
}
