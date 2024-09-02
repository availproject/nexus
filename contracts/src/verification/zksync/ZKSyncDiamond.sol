pragma solidity ^0.8.13;

import "../../interfaces/INexusProofManager.sol";
import "forge-std/console.sol";

contract ZKSyncDiamond {

    INexusProofManager immutable proofManager;
    bytes32 immutable chainId;

    constructor(INexusProofManager _proofManager, bytes32 _chainId) {
        proofManager = _proofManager;
        chainId = _chainId;
    }

    function storedBatchHash(uint256) external returns (bytes32) { 
       return proofManager.getChainState(0, chainId);
    }
}