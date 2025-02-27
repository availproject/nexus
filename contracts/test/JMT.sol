// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import "forge-std/test.sol";
import "../src/NexusProofManager.sol";
import "../src/interfaces/INexusProofManager.sol";
import "../src/mock/ERC20.sol";
import "../src/verification/ethereum/Verifier.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {RiscZeroCheats} from "risc0/test/RiscZeroCheats.sol";

contract EthereumVerifierTest is Test, RiscZeroCheats {
    NexusProofManager proofManager;
    ERC20Token erc20;
    EthereumVerifier verifier;
    RiscZeroVerifierRouter risc0Router;
    IRiscZeroVerifier risc0Verifier;

    bytes32 private constant EMPTY_TRIE_ROOT_HASH =
        0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421;
    bytes32 private constant EMPTY_CODE_HASH =
        0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470;

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
        verifier = new EthereumVerifier(
            INexusProofManager(address(proofManager))
        );
    }

    function testEmptyProof() public {
        uint256 blockNumber = 123;

        bytes32 appid = 0x3655ca59b7d566ae06297c200f98d04da2e8e89812d627bc29297c25db60362d;

        proofManager.updateNexusBlock(
            blockNumber,
            proof,
            journal
        );
        bytes32[] memory siblings;
        NexusProofManager.AccountState memory state = NexusProofManager
            .AccountState(
            0xa01ae135624bda83c2804af67e0931b7069bcf5d9c68a90833460bd17acc6dd5,
            0x0000000000000000000000000000000000000000000000000000000000000000,
        0x171a3a4b1f01465a8bf92f22fdbf8450175f7c963897d6b00e3853a631d79aae,
            0,
            0
            );

        proofManager.updateChainState(blockNumber, siblings, appid, state);
    }

    function testNonEmptyProof() public {
        uint256 blockNumber = 16;
        bytes32 stateRoot = 0x01eabe125b5f4f9ce2b9c3cc3c306fe789bd6f6ef28aa8d2fb2254e1be045e38;
        bytes32 blockHash = 0x01eabe125b5f4f9ce2b9c3cc3c306fe789bd6f6ef28aa8d2fb2254e1be045e38;
        bytes32 appid = 0xa40fb80ad4287819ecda5efac01c74c78d7cb00ca5f9eb5f6c0f19bd09936ac1;

        proofManager.updateNexusBlock(
            blockNumber,
            proof,
            journal
        );
        bytes32[] memory siblings = new bytes32[](1);
        siblings[
            0
        ] = 0x9e09f177a634b05e216d7c69be82589bf33d9c236e157bec7c844c29adda894a;

        NexusProofManager.AccountState memory state = NexusProofManager
            .AccountState(
                0x0000000000000000000000000000000000000000000000000000000000000000,
                0x0000000000000000000000000000000000000000000000000000000000000000,
                0x7d762e1332bba77a369bee1204580472039f972d2d445d7499e814ff485fe76f,
                0,
                0
            );

        proofManager.updateChainState(blockNumber, siblings, appid, state);
    }
}
