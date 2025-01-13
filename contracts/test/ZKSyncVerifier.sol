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

    uint256 blockNumber = 121249;
    bytes32 stateRoot =
        0x640e68e66ba589e11f7006501a79ec882851e42fdb0e11649dd6881df3a5ed9c;
    bytes32 blockHash =
        0x640e68e66ba589e11f7006501a79ec882851e42fdb0e11649dd6881df3a5ed9c;
    bytes32 appid =
        0x1f5ff885ceb5bf1350c4449316b7d703034c1278ab25bcc923d5347645a0117e;

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
