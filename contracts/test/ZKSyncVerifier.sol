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
import {RiscZeroCheats} from "risc0/test/RiscZeroCheats.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";

contract ZKSyncTest is Test, RiscZeroCheats {
    NexusProofManager proofManager;
    ERC20Token erc20;
    StorageProofVerifier verifier;
    RiscZeroVerifierRouter risc0Router;
    IRiscZeroVerifier risc0Verifier;

    uint256 blockNumber = 121249;
    bytes32 stateRoot =
        0x640e68e66ba589e11f7006501a79ec882851e42fdb0e11649dd6881df3a5ed9c;
    bytes32 blockHash =
        0x640e68e66ba589e11f7006501a79ec882851e42fdb0e11649dd6881df3a5ed9c;
    bytes32 appid =
        0x1f5ff885ceb5bf1350c4449316b7d703034c1278ab25bcc923d5347645a0117e;

    // parameters for `updateNexusBlock` function
    // IMP : proof used here is a fake proof. Not a STARK proof
    // This journal is extracted from the nexus geth adapter verification
    bytes journal = hex"f500000056000000c70000008200000064000000130000009c0000003200000025000000460000008f00000029000000a2000000550000000300000014000000f6000000920000001d000000280000005300000062000000be0000003d00000001000000670000009e000000f0000000d70000006f000000550000001a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0860100170000001a0000003a0000004b0000001f00000001000000460000005a0000008b000000f90000002f00000022000000fd000000bf0000008400000050000000170000005f0000007c000000960000003800000097000000d6000000b00000000e0000003800000053000000a600000031000000d70000009a000000ae0000003600000055000000ca00000059000000b7000000d500000066000000ae00000006000000290000007c000000200000000f00000098000000d00000004d000000a2000000e8000000e80000009800000012000000d600000027000000bc00000029000000290000007c00000025000000db00000060000000360000002d00000035e11aa083da4b62f64a80c2b731097e5dcf9b0608a9689cd10b4633d56dcc7a010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    // seal : extracted using `encode_seal` function
    bytes proof =
    hex"0000000025e9678319415568bf12a96447bc492d9bdd0173453b06a22c7f79f85e2dcb89";

    function setUp() public {
        erc20 = new ERC20Token("Avail", "Avail");

        risc0Verifier = deployRiscZeroVerifier();
        risc0Router = new RiscZeroVerifierRouter(msg.sender);
        vm.prank(msg.sender);
        risc0Router.addVerifier(bytes4(0), risc0Verifier);
        proofManager = new NexusProofManager(address(risc0Router));

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
            proof,
            journal
        );
        bytes32[] memory siblings = new bytes32[](3);
        siblings[
            0
        ] = 0xcb105c19f4be44ed55f3c69f6cb75473a17dd4f005ffb3ba06086c8e8208c1fe;
        siblings[
            1
        ] = 0x0000000000000000000000000000000000000000000000000000000000000000;
        siblings[
            2
        ] = 0x0000000000000000000000000000000000000000000000000000000000000000;
        NexusProofManager.AccountState memory state = NexusProofManager
            .AccountState(
                0xd48b446b8785f787021914f1bea9d6ec04e9480806e56acf9ee17f1bb23bad48,
                0x84b2b689fba40661e61ed5e0df1ab3bc989832b218814f363249774c3a32102f,
                0x8fbfdcd52c25ef8a2841f83a3adf19b1e0bee8b3ee7b4eff04e97319436af334,
                121248,
                660
            );

        proofManager.updateChainState(blockNumber, siblings, appid, state);

        bytes32[] memory dynamicPath = new bytes32[](15);
        dynamicPath[
            0
        ] = 0xc2792a032a5dcdbf741731810685dc60d31559df51b95d5b715285697242954a;
        dynamicPath[
            1
        ] = 0xf9727f1b8a07653de7bb30692db15f5ce2afa51fe7ffce8545f68c29960ebd4a;
        dynamicPath[
            2
        ] = 0xef14b47a044ee399fd4451d464a8b6b1b40c0a14bacfedfa0f0cf441755ddaf7;
        dynamicPath[
            3
        ] = 0xfa5f2b69b20b51dd71dfece0e1dcb3c436101a8ca204b44cc6419d3f5c17ac7b;
        dynamicPath[
            4
        ] = 0x216728456e979189d34149ae1b3d2a8430134f1981d10ca84374c32204b0005a;
        dynamicPath[
            5
        ] = 0x4ef45453f4f99186929756cc6677530541e0d62e7a3ac1436e42d6b02e876bb2;
        dynamicPath[
            6
        ] = 0x81f9e053944516b399589b36ee9d4fa25664327154f74d6f3a98b4c1f3ba3e90;
        dynamicPath[
            7
        ] = 0x976f15832bfc9ea6a09053ff51d14b9e174ae9dbc8f22d243e7c4f144be8bed3;
        dynamicPath[
            8
        ] = 0x4ff77af28422b94f8d54241674f8f81cdd2b35f01d1c548b9606b4b941565e02;
        dynamicPath[
            9
        ] = 0x16e7429492f8db53f154ab50ad43959dd011d4de0864af44bec6b4bd75a4a09e;
        dynamicPath[
            10
        ] = 0x078ab2581c8a5b380c48bf067199876377e3a06dfd2248b57e60a9df501977f6;
        dynamicPath[
            11
        ] = 0xb61730f6a498d4a081187bcdf924ba4588d595aca7228b03f38ed631001fc6ac;
        dynamicPath[
            12
        ] = 0x72b06356414b0a3f5fed00f9453e0565238d2ffecc000820821714747f32765b;
        dynamicPath[
            13
        ] = 0xf2c9dc3dbf1e7a87aae33c95eea8c8e31ccdb5e1eaaa36ccec0e0e77352d6856;
        dynamicPath[
            14
        ] = 0x90cfcac4642304a3d87b0a20c4e0961b07e3a7a9ebb1ec221fe9eac7bff90342;

        StorageProof memory proof = StorageProof(
            660,
            0x9a03a545A60263216c4310Be05C34B71C170903A,
            0x0000000000000000000000000000000000000000000000000000000000000001,
            dynamicPath,
            14698
        );
        assert(
            verifier.verify(
                proof,
                0xcef9eeeac760226b597a2b40094bd64f19121e98613c58b193167c303344b15f
            )
        );
    }
}
