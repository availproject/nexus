// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import "forge-std/test.sol";
import {MailboxMessage as NexusReceipt, VerifierInfo} from "../src/interfaces/INexusMailbox.sol";
import "../src/NexusProofManager.sol";
import "../src/interfaces/INexusProofManager.sol";
import "../src/mock/ERC20.sol";
import "../src/mock/FallbackContract.sol";
import "../src/verification/zksync/StorageProof.sol";
import "../src/verification/zksync/SparseMerkleTree.sol";
import "../src/verification/zksync/VerifierWrapper.sol";
import "../src/verification/zksync/ZKSyncNexusManagerRouter.sol";

import "./NexusMailboxWrapper.sol";

contract MailBoxTest is Test {
    NexusMailboxWrapper mailbox;
    NexusProofManager proofManager;
    ERC20Token erc20;
    VerifierWrapper wrapper;

    uint256 targetnexusAppID = 137;
    bytes32 appIdDestination =
        0x3655ca59b7d566ae06297c200f98d04da2e8e89812d627bc29297c25db60362d;
    function setUp() public {
        mailbox = new NexusMailboxWrapper();
        mailbox.initialize(appIdDestination);
        erc20 = new ERC20Token("Avail", "Avail");
        proofManager = new NexusProofManager();
        SparseMerkleTree smt = new SparseMerkleTree();
        ZKSyncNexusManagerRouter zksyncDiamond = new ZKSyncNexusManagerRouter(
            INexusProofManager(address(proofManager)),
            appIdDestination
        );
        wrapper = new VerifierWrapper(
            IZKSyncNexusManagerRouter(address(zksyncDiamond)),
            smt
        );
        mailbox.addOrUpdateWrapper(
            bytes32(targetnexusAppID),
            VerifierInfo(
                INexusVerifierWrapper(address(wrapper)),
                0x9a03a545A60263216c4310Be05C34B71C170903A
            )
        );
    }

    function testSendMessage() public {
        uint256 length = 1;
        bytes32[] memory nexusAppIDTo = new bytes32[](length);
        nexusAppIDTo[0] = bytes32(targetnexusAppID);
        address[] memory to = new address[](length);
        to[0] = address(0);
        bytes memory data = bytes("test");
        bytes32 nexusAppID = mailbox.nexusAppID();
        uint256 mailboxNonce = 1;
        mailbox.sendMessage(nexusAppIDTo, to, mailboxNonce, data);

        NexusReceipt memory receipt = NexusReceipt({
            nexusAppIDFrom: nexusAppID,
            nexusAppIDTo: nexusAppIDTo,
            data: data,
            from: address(this),
            to: to,
            nonce: mailboxNonce
        });

        bytes32 receiptHash = keccak256(abi.encode(receipt));

        assertEq(mailbox.messages(receiptHash), true);
    }

    function testReceiveReceipt() public {
        mailbox.addOrUpdateWrapper(
            bytes32(targetnexusAppID),
            VerifierInfo(
                INexusVerifierWrapper(address(wrapper)),
                0x6bc15F6C8abD245812C7eC650D4586b9B52Ae546
            )
        );
        uint256 key = 0xfaaf1897615a4d5824a81780f33dd422a304cae5e7b14f0f9215d1a3deeea9e2;

        bytes32 value = 0x7fc8e033e28402e82ae3c4a4e6d7d02ab3941505362bdb58c429a2ffc9870802;
        bytes32[] memory dynamicPath = new bytes32[](9);
        dynamicPath[0] = bytes32(
            0xba5325838c32aa67257f995767d0a51bb9652e86b162dcc8fbb43b15cc5c7ae5
        );
        dynamicPath[1] = bytes32(
            0x01de01ebbdc33833eb4e9049fa9bb20f0268737312999115a14d553c661a3b6c
        );
        dynamicPath[2] = bytes32(
            0xc89cb40d1ae178bbc7e18800b0aa460f53a070d710c4c70ebc8731f0d3812e22
        );
        dynamicPath[3] = bytes32(
            0xc631fffdfdbc27ed0e4f61bc50b799ee0d9b67d5e9cac886e703144e9572712d
        );
        dynamicPath[4] = bytes32(
            0x4e1e5eb29f3378179f87112827a22ce510fd6b80b11d4ea70b8ca50414e1e67b
        );
        dynamicPath[5] = bytes32(
            0xdd2ee4dcfdab21b5746de659fc8742cf5671520826ee90216e142b165c26eb3f
        );
        dynamicPath[6] = bytes32(
            0xe01a1ba6f8acab9e567849199d1af48b883532a642724b269d824745f07d959a
        );
        dynamicPath[7] = bytes32(
            0xbd4efdde3e1211ff26d4549887187e6b4ab232b718f4902e5e7ccf00493e7b68
        );
        dynamicPath[8] = bytes32(
            0xdc9a374febf417a247dbf3974ca6b39344266105d9c93f32a9fa2301e6d19a98
        );

        StorageProof memory proof = StorageProof(
            123,
            0x6bc15F6C8abD245812C7eC650D4586b9B52Ae546, // account
            value, // value
            dynamicPath,
            581
        );

        bytes memory encoding = abi.encode(proof);

        mailbox.updateSendMessages(key, value);

        uint256 length = 1;
        bytes32[] memory nexusAppIDTo = new bytes32[](length);
        nexusAppIDTo[0] = bytes32(targetnexusAppID);
        address[] memory to = new address[](length);
        to[0] = address(0);
        bytes memory data = bytes("test");
        uint256 mailboxNonce = 1;
        mailbox.sendMessage(nexusAppIDTo, to, mailboxNonce, data);

        NexusReceipt memory receipt = NexusReceipt({
            nexusAppIDFrom: mailbox.nexusAppID(),
            nexusAppIDTo: nexusAppIDTo,
            data: data,
            from: address(this),
            to: to,
            nonce: mailboxNonce
        });

        mailbox.checkVerificationOfEncoding(
            0,
            receipt,
            bytes32(targetnexusAppID),
            value,
            encoding
        );
    }

    function testReceiveReceiptCallback() public {
        uint256 blockNumber = 121249;
        bytes32 stateRoot = 0x640e68e66ba589e11f7006501a79ec882851e42fdb0e11649dd6881df3a5ed9c;
        bytes32 blockHash = 0x640e68e66ba589e11f7006501a79ec882851e42fdb0e11649dd6881df3a5ed9c;
        bytes32 appid = 0x1f5ff885ceb5bf1350c4449316b7d703034c1278ab25bcc923d5347645a0117e;
        uint128 chainBlockNumber = 660;

        uint256 key = 0xcef9eeeac760226b597a2b40094bd64f19121e98613c58b193167c303344b15f;
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
                chainBlockNumber
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

        bytes memory encoding = abi.encode(proof);

        uint256 length = 1;
        bytes32[] memory nexusAppIDTo = new bytes32[](length);
        nexusAppIDTo[0] = bytes32(appIdDestination);
        address[] memory to = new address[](length);
        FallbackContract fallbackContract = new FallbackContract();
        to[0] = address(fallbackContract);
        bytes memory data = bytes("test");
        uint256 mailboxNonce = 1;

        NexusReceipt memory receipt = NexusReceipt({
            nexusAppIDFrom: bytes32(targetnexusAppID),
            nexusAppIDTo: nexusAppIDTo,
            data: data,
            from: address(this),
            to: to,
            nonce: mailboxNonce
        });
        vm.expectCall(
            address(fallbackContract),
            abi.encodeWithSignature(
                "onNexusMessage(bytes32,address,bytes,uint256)",
                bytes32(targetnexusAppID),
                address(this),
                data,
                mailboxNonce
            )
        );
        mailbox.receiveMessage(chainBlockNumber, receipt, encoding);
    }

    function testSortingAlgorithm() public view {
        uint256 length = 5;
        bytes32[] memory nexusAppIDTo = new bytes32[](length);
        nexusAppIDTo[0] = bytes32(targetnexusAppID);
        nexusAppIDTo[1] = bytes32(targetnexusAppID - 1);
        nexusAppIDTo[2] = bytes32(targetnexusAppID + 1);
        nexusAppIDTo[3] = bytes32(targetnexusAppID + 2);
        nexusAppIDTo[4] = bytes32(targetnexusAppID - 2);

        address[] memory to = new address[](length);
        to[0] = address(0);
        to[1] = vm.addr(1);
        to[2] = vm.addr(2);
        to[3] = vm.addr(3);
        to[4] = vm.addr(4);

        (nexusAppIDTo, to) = mailbox.sortWrapper(
            nexusAppIDTo,
            to,
            0,
            int256(length - 1)
        );

        assertEq(nexusAppIDTo[0], bytes32(targetnexusAppID - 2));
        assertEq(nexusAppIDTo[1], bytes32(targetnexusAppID - 1));
        assertEq(nexusAppIDTo[2], bytes32(targetnexusAppID));
        assertEq(nexusAppIDTo[3], bytes32(targetnexusAppID + 1));
        assertEq(nexusAppIDTo[4], bytes32(targetnexusAppID + 2));

        assertEq(to[0], vm.addr(4));
        assertEq(to[1], vm.addr(1));
        assertEq(to[2], address(0));
        assertEq(to[3], vm.addr(2));
        assertEq(to[4], vm.addr(3));
    }

    function testSearchAlgorithm() public view {
        uint256 length = 5;
        bytes32[] memory nexusAppIDTo = new bytes32[](length);
        bytes32 nexusAppID = mailbox.nexusAppID();
        nexusAppIDTo[0] = nexusAppID;
        nexusAppIDTo[1] = bytes32(targetnexusAppID - 1);
        nexusAppIDTo[2] = bytes32(targetnexusAppID + 1);
        nexusAppIDTo[3] = bytes32(targetnexusAppID + 2);
        nexusAppIDTo[4] = bytes32(targetnexusAppID - 2);

        address[] memory to = new address[](length);
        to[0] = vm.addr(2);
        to[1] = vm.addr(1);
        to[2] = address(0);
        to[3] = vm.addr(3);
        to[4] = vm.addr(4);

        (nexusAppIDTo, to) = mailbox.sortWrapper(
            nexusAppIDTo,
            to,
            0,
            int256(length - 1)
        );

        address toAddr = mailbox.searchWrapper(nexusAppIDTo, to);
        assertEq(toAddr, vm.addr(2));
    }
}
