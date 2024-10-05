// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import "forge-std/test.sol";
import {MailboxMessage as NexusReceipt} from "../src/interfaces/INexusMailbox.sol";
import "../src/NexusProofManager.sol";
import "../src/interfaces/INexusProofManager.sol";
import "../src/mock/ERC20.sol";
import "../src/verification/zksync/StorageProof.sol";
import "../src/verification/zksync/SparseMerkleTree.sol";
import "../src/verification/zksync/VerifierWrapper.sol";
import "../src/verification/zksync/ZKSyncNexusManagerRouter.sol";

import "./NexusMailboxWrapper.sol";

contract MailBoxTest is Test {
    NexusMailboxWrapper mailbox;
    NexusProofManager proofManager;
    ERC20Token erc20;

    bytes32 appid =
        0x3655ca59b7d566ae06297c200f98d04da2e8e89812d627bc29297c25db60362d;
    uint256 targetnexusAppId = 137;

    function setUp() public {
        mailbox = new NexusMailboxWrapper();
        mailbox.initialize();
        erc20 = new ERC20Token("Avail", "Avail");
        proofManager = new NexusProofManager();
        SparseMerkleTree smt = new SparseMerkleTree();
        ZKSyncNexusManagerRouter zksyncDiamond = new ZKSyncNexusManagerRouter(
            INexusProofManager(address(proofManager)),
            appid
        );
        VerifierWrapper wrapper = new VerifierWrapper(
            IZKSyncNexusManagerRouter(address(zksyncDiamond)),
            smt
        );
        mailbox.addOrUpdateWrapper(bytes32(targetnexusAppId), wrapper);
    }

    function testSendMessage() public {
        uint256 length = 1;
        bytes32[] memory nexusAppIdTo = new bytes32[](length);
        nexusAppIdTo[0] = bytes32(targetnexusAppId);
        address[] memory to = new address[](length);
        to[0] = address(0);
        bytes memory data = bytes("test");
        bytes32 nexusAppId = mailbox.nexusAppId();
        uint256 mailboxNonce = 1;
        mailbox.sendMessage(nexusAppIdTo, to, mailboxNonce, data);

        NexusReceipt memory receipt = NexusReceipt({
            nexusAppIdFrom: nexusAppId,
            nexusAppIdTo: nexusAppIdTo,
            data: data,
            from: address(this),
            to: to,
            nonce: mailboxNonce
        });

        bytes32 receiptHash = keccak256(abi.encode(receipt));
        bytes32 key = keccak256(abi.encode(address(this), receiptHash));

        assertEq(mailbox.messages(key), receiptHash);
    }

    function testReceiveReceipt() public {
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
            key, // key
            value, // value
            dynamicPath,
            581
        );

        bytes memory encoding = abi.encode(proof);

        mailbox.updateSendMessages(key, value);

        uint256 length = 1;
        bytes32[] memory nexusAppIdTo = new bytes32[](length);
        nexusAppIdTo[0] = bytes32(targetnexusAppId);
        address[] memory to = new address[](length);
        to[0] = address(0);
        bytes memory data = bytes("test");
        uint256 mailboxNonce = 1;
        mailbox.sendMessage(nexusAppIdTo, to, mailboxNonce, data);

        NexusReceipt memory receipt = NexusReceipt({
            nexusAppIdFrom: mailbox.nexusAppId(),
            nexusAppIdTo: nexusAppIdTo,
            data: data,
            from: address(this),
            to: to,
            nonce: mailboxNonce
        });

        mailbox.checkVerificationOfEncoding(
            0,
            receipt,
            bytes32(targetnexusAppId),
            value,
            encoding
        );
    }

    function testSortingAlgorithm() public view {
        uint256 length = 5;
        bytes32[] memory nexusAppIdTo = new bytes32[](length);
        nexusAppIdTo[0] = bytes32(targetnexusAppId);
        nexusAppIdTo[1] = bytes32(targetnexusAppId - 1);
        nexusAppIdTo[2] = bytes32(targetnexusAppId + 1);
        nexusAppIdTo[3] = bytes32(targetnexusAppId + 2);
        nexusAppIdTo[4] = bytes32(targetnexusAppId - 2);

        address[] memory to = new address[](length);
        to[0] = address(0);
        to[1] = vm.addr(1);
        to[2] = vm.addr(2);
        to[3] = vm.addr(3);
        to[4] = vm.addr(4);

        (nexusAppIdTo, to) = mailbox.sortWrapper(
            nexusAppIdTo,
            to,
            0,
            int256(length - 1)
        );

        assertEq(nexusAppIdTo[0], bytes32(targetnexusAppId - 2));
        assertEq(nexusAppIdTo[1], bytes32(targetnexusAppId - 1));
        assertEq(nexusAppIdTo[2], bytes32(targetnexusAppId));
        assertEq(nexusAppIdTo[3], bytes32(targetnexusAppId + 1));
        assertEq(nexusAppIdTo[4], bytes32(targetnexusAppId + 2));

        assertEq(to[0], vm.addr(4));
        assertEq(to[1], vm.addr(1));
        assertEq(to[2], address(0));
        assertEq(to[3], vm.addr(2));
        assertEq(to[4], vm.addr(3));
    }

    function testSearchAlgorithm() public view {
        uint256 length = 5;
        bytes32[] memory nexusAppIdTo = new bytes32[](length);
        bytes32 nexusAppId = mailbox.nexusAppId();
        nexusAppIdTo[0] = nexusAppId;
        nexusAppIdTo[1] = bytes32(targetnexusAppId - 1);
        nexusAppIdTo[2] = bytes32(targetnexusAppId + 1);
        nexusAppIdTo[3] = bytes32(targetnexusAppId + 2);
        nexusAppIdTo[4] = bytes32(targetnexusAppId - 2);

        address[] memory to = new address[](length);
        to[0] = vm.addr(2);
        to[1] = vm.addr(1);
        to[2] = address(0);
        to[3] = vm.addr(3);
        to[4] = vm.addr(4);

        (nexusAppIdTo, to) = mailbox.sortWrapper(
            nexusAppIdTo,
            to,
            0,
            int256(length - 1)
        );

        address toAddr = mailbox.searchWrapper(nexusAppIdTo, to);
        assertEq(toAddr, vm.addr(2));
    }
}
