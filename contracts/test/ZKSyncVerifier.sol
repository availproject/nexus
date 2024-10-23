// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

// This contract is based on the original work found in https://github.com/QEDK/jmt/blob/master/src/JellyfishMerkleTreeVerifier.sol
// Licensed under the Apache License, Version 2.0.

import "forge-std/test.sol";
import "../src/NexusProofManager.sol";
import "../src/interfaces/INexusProofManager.sol";
import "../src/mock/ERC20.sol";
import "../src/verification/zksync/StorageProof.sol";
import "../src/verification/zksync/SparseMerkleTree.sol";
import "../src/verification/zksync/ZKSyncNexusManagerRouter.sol";

contract ZKSyncTest is Test {
    NexusProofManager proofManager;
    ERC20Token erc20;
    StorageProofVerifier verifier;

    bytes32[] dynamicPath;
    uint256 blockNumber = 123;
    bytes32 stateRoot =
        0x118eabaae552430cdecf445736d2e57c5dbcf70c1688f053e70f0c3a6a80411f;
    bytes32 blockHash =
        0x118eabaae552430cdecf445736d2e57c5dbcf70c1688f053e70f0c3a6a80411f;
    bytes32 appid =
        0x3655ca59b7d566ae06297c200f98d04da2e8e89812d627bc29297c25db60362d;

    function setUp() public {
        erc20 = new ERC20Token("Avail", "Avail");
        proofManager = new NexusProofManager();
        SparseMerkleTree smt = new SparseMerkleTree();
        ZKSyncNexusManagerRouter zksyncDiamond = new ZKSyncNexusManagerRouter(
            INexusProofManager(address(proofManager)),
            appid
        );
        verifier = new StorageProofVerifier(
            IZKSyncNexusManagerRouter(address(zksyncDiamond)),
            smt
        );
    }

    function testStorageProof() public {
        proofManager.updateNexusBlock(
            blockNumber,
            NexusProofManager.NexusBlock(stateRoot, blockHash)
        );
        bytes32[] memory siblings;
        NexusProofManager.AccountState memory state = NexusProofManager
            .AccountState(
                0x509248c5752f1898dfea0887e7617a84631e749a404a25e976c6d3883c789b3b,
                0xd62c0e6039b3b76b0c70301de2dee44f1f8d1335e7df9bd26fc3bdb6f33a2574,
                0x378f4888b185704cb8c8e86792838c2fed7f7d4bd58cd9e66b34050a9c42aad1,
                570,
                123
            );

        proofManager.updateChainState(blockNumber, siblings, appid, state);

        dynamicPath.push(
            bytes32(
                0xba5325838c32aa67257f995767d0a51bb9652e86b162dcc8fbb43b15cc5c7ae5
            )
        );
        dynamicPath.push(
            bytes32(
                0x01de01ebbdc33833eb4e9049fa9bb20f0268737312999115a14d553c661a3b6c
            )
        );
        dynamicPath.push(
            bytes32(
                0xc89cb40d1ae178bbc7e18800b0aa460f53a070d710c4c70ebc8731f0d3812e22
            )
        );
        dynamicPath.push(
            bytes32(
                0xc631fffdfdbc27ed0e4f61bc50b799ee0d9b67d5e9cac886e703144e9572712d
            )
        );
        dynamicPath.push(
            bytes32(
                0x4e1e5eb29f3378179f87112827a22ce510fd6b80b11d4ea70b8ca50414e1e67b
            )
        );
        dynamicPath.push(
            bytes32(
                0xdd2ee4dcfdab21b5746de659fc8742cf5671520826ee90216e142b165c26eb3f
            )
        );
        dynamicPath.push(
            bytes32(
                0xe01a1ba6f8acab9e567849199d1af48b883532a642724b269d824745f07d959a
            )
        );
        dynamicPath.push(
            bytes32(
                0xbd4efdde3e1211ff26d4549887187e6b4ab232b718f4902e5e7ccf00493e7b68
            )
        );
        dynamicPath.push(
            bytes32(
                0xdc9a374febf417a247dbf3974ca6b39344266105d9c93f32a9fa2301e6d19a98
            )
        );

        StorageProof memory proof = StorageProof(
            123,
            0x6bc15F6C8abD245812C7eC650D4586b9B52Ae546,
            0x7fc8e033e28402e82ae3c4a4e6d7d02ab3941505362bdb58c429a2ffc9870802,
            dynamicPath,
            581
        );
        assert(
            verifier.verify(
                proof,
                0xfaaf1897615a4d5824a81780f33dd422a304cae5e7b14f0f9215d1a3deeea9e2
            )
        );
    }
}
