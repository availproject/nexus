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

contract ZKSyncTest is Test, RiscZeroCheats {
    NexusProofManager proofManager;
    ERC20Token erc20;
    StorageProofMock verifier;
    RiscZeroVerifierRouter risc0Router;
    IRiscZeroVerifier risc0Verifier;

    uint256 blockNumber = 121249;
    bytes32 appid = 0x3655ca59b7d566ae06297c200f98d04da2e8e89812d627bc29297c25db60362d;

    // parameters for `updateNexusBlock` function
    // IMP : proof used here is a fake proof. Not a STARK proof
    // This journal is extracted from the nexus geth adapter verification
    bytes journal =
        hex"d8000000c3000000c400000079000000f300000055000000510000001a0000000a000000e70000009e000000f60000005a0000005a0000009f000000b7000000e6000000270000000f00000060000000ab00000043000000d6000000a9000000e9000000df000000d00000000a0000002c0000004600000059000000d000000071000000020000003d000000940000009b000000ca00000040000000e30000005500000002000000800000002f000000dc000000ff00000046000000b40000008e00000056000000cc000000d30000002800000010000000ef000000a9000000b600000098000000380000003f000000a700000052000000ec000000f700000072833400d8000000c3000000c400000079000000f300000055000000510000001a0000000a000000e70000009e000000f60000005a0000005a0000009f000000b7000000e6000000270000000f00000060000000ab00000043000000d6000000a9000000e9000000df000000d00000000a0000002c0000004600000059000000d00000003600000055000000ca00000059000000b7000000d500000066000000ae00000006000000290000007c000000200000000f00000098000000d00000004d000000a2000000e8000000e80000009800000012000000d600000027000000bc00000029000000290000007c00000025000000db00000060000000360000002d00000035e11aa083da4b62f64a80c2b731097e5dcf9b0608a9689cd10b4633d56dcc7a010000008d0000001d000000b1000000e400000049000000490000005a00000018000000560000007c000000d900000036000000970000002b000000240000003a0000003a00000034000000ee00000089000000eb000000b2000000b0000000a9000000740000004400000048000000bb0000004b000000b20000005200000064000000";
    // seal : extracted using `encode_seal` function
    bytes proof = hex"000000009eaa7b47d953bb850cbf8957bcacb2bd9a943120b3ee4c8494c9f21fba814b7f";

    function setUp() public {
        erc20 = new ERC20Token("Avail", "Avail");

        risc0Verifier = deployRiscZeroVerifier();
        risc0Router = new RiscZeroVerifierRouter(msg.sender);
        vm.prank(msg.sender);
        risc0Router.addVerifier(bytes4(0), risc0Verifier);
        proofManager = new NexusProofManager(address(risc0Router), ImageID.NEXUS_RUNTIME_ID);

        SparseMerkleTree smt = new SparseMerkleTree();
        ZKSyncNexusManagerRouter zksyncDiamond =
            new ZKSyncNexusManagerRouter(INexusProofManager(address(proofManager)), appid);
        verifier = new StorageProofMock(IZKSyncNexusManagerRouter(address(zksyncDiamond)), smt);
    }

    function testStorageProof() public {
        proofManager.updateNexusBlock(blockNumber, proof, journal);
        bytes32[] memory siblings;
        Structs.AppState memory state = Structs.AppState(
            0xa01ae135624bda83c2804af67e0931b7069bcf5d9c68a90833460bd17acc6dd5,
            0x0000000000000000000000000000000000000000000000000000000000000000,
            0x30c23598430f6c4eb3d583a394240b281936dfc243e2417b4e8c9017a9679c56,
            2,
            3441521
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
