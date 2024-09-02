pragma solidity ^0.8.13;

import {JellyfishMerkleTreeVerifier} from "./lib/JellyfishMerkleTreeVerifier.sol";

contract NexusProofManager  {
    uint256 public latestNexusBlockNumber = 0;

    struct NexusBlock {
        bytes32 stateRoot;
        bytes32 blockHash;
    }

    mapping(uint256 => NexusBlock) public nexusBlock;
    mapping(bytes32=>uint256) public nexusAppIDToLatestBlockNumber;
    mapping(bytes32 => mapping(uint256 => bytes32)) public nexusAppIDToState;

    struct AccountState { 
        bytes32 statementDigest;
        bytes32 stateRoot;
        bytes32 startNexusHash;
        uint128 lastProofHeight;
        uint128 height;
    }

    constructor(bytes32 nexusAppID) {}

    // nexus state root
    // updated when we verify the zk proof and then st block updated
    function updateNexusBlock(uint256 blockNumber, NexusBlock calldata latestNexusBlock) external  {
        require(blockNumber > latestNexusBlockNumber, "Block already updated");
        // TODO: verify a zk proof from nexus
        nexusBlock[blockNumber] = latestNexusBlock;
        latestNexusBlockNumber = blockNumber;
    }


    function updateChainState(uint256 nexusBlockNumber, bytes32[] calldata  siblings, bytes32 key,  AccountState calldata accountState) external {
        bytes32 valueHash = sha256(abi.encode(accountState.statementDigest, accountState.stateRoot, accountState.startNexusHash, accountState.lastProofHeight,accountState.height));
        JellyfishMerkleTreeVerifier.Leaf memory leaf = JellyfishMerkleTreeVerifier.Leaf({
        addr: key,
        valueHash: valueHash
        });

        JellyfishMerkleTreeVerifier.Proof memory proof = JellyfishMerkleTreeVerifier.Proof({
            leaf: leaf,
            siblings: siblings
        });

        verifyRollupState(nexusBlock[nexusBlockNumber].stateRoot , proof, leaf);   
        
        require(nexusAppIDToLatestBlockNumber[key]<accountState.height,"Old block number");
        nexusAppIDToLatestBlockNumber[key] = accountState.height;
        nexusAppIDToState[key][accountState.height] = accountState.stateRoot;
    }


    function verifyRollupState(bytes32 root, JellyfishMerkleTreeVerifier.Proof memory proof, JellyfishMerkleTreeVerifier.Leaf memory leaf) pure public {
        bool verify = JellyfishMerkleTreeVerifier.verifyProof(root, leaf, proof);
        require(verify,"Invalid leaf against nexus state root");
    }


    function getChainState(uint256 blockNumber, bytes32 nexusAppID) external view returns(bytes32) {
        if (blockNumber == 0 ){ 
            return nexusAppIDToState[nexusAppID][nexusAppIDToLatestBlockNumber[nexusAppID]];
        }
        else {
              require(blockNumber <= nexusAppIDToLatestBlockNumber[nexusAppID]);
              return nexusAppIDToState[nexusAppID][blockNumber];
        }
    }
}