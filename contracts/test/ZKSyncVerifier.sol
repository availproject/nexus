// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

// This contract is based on the original work found in https://github.com/QEDK/jmt/blob/master/src/JellyfishMerkleTreeVerifier.sol
// Licensed under the Apache License, Version 2.0.

import "forge-std/test.sol";
import "../src/NexusProofManager.sol";
import {Structs} from "../src/lib/Structs.sol";
import "../src/interfaces/INexusProofManager.sol";
import "../src/mock/ERC20.sol";
import "../src/verification/zksync/StorageProofMock.sol";
import "../src/verification/zksync/SparseMerkleTree.sol";
import "../src/verification/zksync/ZKSyncNexusManagerRouter.sol";
import {RiscZeroCheats} from "risc0/test/RiscZeroCheats.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {ImageID} from "../src/NexusProverImageID.sol";
import {IVectorx} from "../src/interfaces/IVectorx.sol";

contract ZKSyncTest is Test, RiscZeroCheats {
    NexusProofManager proofManager;
    ERC20Token erc20;
    StorageProofMock verifier;
    RiscZeroVerifierRouter risc0Router;
    IRiscZeroVerifier risc0Verifier;
    IVectorx vectorX;

    uint256 blockNumber = 121249;
    bytes32 appid = 0x3655ca59b7d566ae06297c200f98d04da2e8e89812d627bc29297c25db60362d;

    // parameters for `updateNexusBlock` function
    // IMP : proof used here is a fake proof. Not a STARK proof
    // This journal is nexus proof which is generated through mock elf proving.
    bytes journal =
        hex"e2d612283fc162a34ec4cae21335f10efad0234e23f281aae67215b0edb0dca178960bb207f285632596f8932b98117adec87241e5c09346cbc5637ede0df7c4e7cce75ae1f47fab781b78e332473d7d9a43c713b1f533b06eda6b38e7c4dd04c2df954f375c6273ad0a38ccff49bc230daec7cd9a4418b8e247f6ff7ecd87bf0f000000";
    // seal : extracted using `encode_seal` function
    bytes proof = hex"fffffffffa454345a5049d6b0e9d44d59d788b07f0b04b7afc7337ba113be0e28d684895";

    function setUp() public {
        erc20 = new ERC20Token("Avail", "Avail");

        risc0Verifier = deployRiscZeroVerifier();

        // TODO : need to change this with actual implementation when writing tests
        vectorX = IVectorx(msg.sender);

        risc0Router = new RiscZeroVerifierRouter(msg.sender);
        vm.prank(msg.sender);
        risc0Router.addVerifier(bytes4(0), risc0Verifier);
        proofManager = new NexusProofManager(address(risc0Router), ImageID.NEXUS_RUNTIME_ID, address(vectorX));

        SparseMerkleTree smt = new SparseMerkleTree();
        ZKSyncNexusManagerRouter zksyncDiamond =
            new ZKSyncNexusManagerRouter(INexusProofManager(address(proofManager)), appid);
        verifier = new StorageProofMock(IZKSyncNexusManagerRouter(address(zksyncDiamond)), smt);
    }

    function testStorageProof() public {
        proofManager.updateNexusBlock(blockNumber, proof, journal);
        bytes32[] memory siblings;
        Structs.AppState memory state = Structs.AppState(
            0xb0abae3a180de2034da3ce8084bc0afcbea6a5f64efb4bb3c9b9c4a841fdaa62,
            0x78960bb207f285632596f8932b98117adec87241e5c09346cbc5637ede0df7c4,
            0xadb3f87a06a0f912592ba7c77aa784a383c80486e75969b48f8cb5453bcc91ba,
            1,
            398570
        );

        proofManager.updateChainState(blockNumber, siblings, appid, state);

        bytes32[] memory dynamicPath = new bytes32[](15);
        dynamicPath[0] = 0x0f543536a5c50f86b1b9790051824be684b91cd22daa901755f6de2f3a597b60;
        dynamicPath[1] = 0x9b754ff40db4b917183d8a752d1d891c2405b7b0dee15ab76626352ad6c8e5eb;
        dynamicPath[2] = 0x5188bf4a51f7b8f6c00025561e7f589c1ced57779749fe08ec4fd651642a7875;
        dynamicPath[3] = 0x8eb612ba6d63a9ed3f9931f557139e5abf02fb53e15d5bbd0179ea0fd24bca4e;
        dynamicPath[4] = 0x9ed5b49f326c187588281fc2dc2b4bacf4875f61629b34d70b7bc51cab5fc462;
        dynamicPath[5] = 0xfc6030c5df6f96e02abeb8b32343bb315ca7b5c23c1286b2f728a91ca5efa440;
        dynamicPath[6] = 0xd33f3439037a4c9a290fe7aa6403a6719deea137bb1a3bf2353834ab6d6b3fc3;
        dynamicPath[7] = 0x5de3191254d39be06956967aaf3847ab2f2d9c90eb92316aefaf9f64703ed814;
        dynamicPath[8] = 0x83d6b373dd271f119a27a67e84cf95ec3074a4abcd4e2fcd2c50e30a7145671e;
        dynamicPath[9] = 0xe5f51e9b3b8e479ac7378fbb652d099b410e16f307d8b1c1df04c06eb73c3e02;
        dynamicPath[10] = 0xea6ad02f6081088896d58d00e3cf6ac6fae74929d40fc7f17c75e9601cb6e20f;
        dynamicPath[11] = 0x56c6c895ab1a717811a7b5d70ed7b7507159258b9c022631bf278543404bb61f;
        dynamicPath[12] = 0xa243ba9ff58329ea3b9aaf648788005232b7b65b8d5e21648637983378e25494;
        dynamicPath[13] = 0xfb9a3a130e03da5f79b4acae017e4d4e0b13f2886f26e165bd56e746c31d2b04;
        dynamicPath[14] = 0xb7ae54987f5828d2dd8e942bab07aa67b1117395255d595aee146489cadf8640;

        StorageProof memory proof = StorageProof(
            660,
            0x9a03a545A60263216c4310Be05C34B71C170903A,
            0x0000000000000000000000000000000000000000000000000000000000000000,
            dynamicPath,
            0
        );
        // TODO : need to remove the mock verifier and implement the actual verifier
        assert(verifier.verify(proof, 0xcef9eeeac760226b597a2b40094bd64f19121e98613c58b193167c303344b15f));
    }
}
