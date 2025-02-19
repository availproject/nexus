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
    bytes journal = hex"690000000d0000006e0000005a000000c9000000d80000009f000000aa0000008f0000002100000009000000e800000066000000bc000000b5000000ac0000004600000056000000b9000000de0000007600000050000000e30000008100000042000000c800000039000000f80000002200000008000000060000009a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040420f00170000001a0000003a0000004b0000001f00000001000000460000005a0000008b000000f90000002f00000022000000fd000000bf0000008400000050000000170000005f0000007c000000960000003800000097000000d6000000b00000000e0000003800000053000000a600000031000000d70000009a000000ae0000003600000055000000ca00000059000000b7000000d500000066000000ae00000006000000290000007c000000200000000f00000098000000d00000004d000000a2000000e8000000e80000009800000012000000d600000027000000bc00000029000000290000007c00000025000000db00000060000000360000002d00000099b31880a6a2af6a220b5798455934c409b231cabfe15a39d11ee9aa715849bc010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
    // seal : extracted using `encode_seal` function
    bytes proof =
    hex"00000000c931f89898189486e66b74613e2412690df4d6193bacda376fb3ccd686a4f97c";

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

        bytes32 stateRoot = 0x118eabaae552430cdecf445736d2e57c5dbcf70c1688f053e70f0c3a6a80411f;
        bytes32 blockHash = 0x118eabaae552430cdecf445736d2e57c5dbcf70c1688f053e70f0c3a6a80411f;
        bytes32 appid = 0x3655ca59b7d566ae06297c200f98d04da2e8e89812d627bc29297c25db60362d;

        proofManager.updateNexusBlock(
            blockNumber,
            NexusProofManager.NexusBlock(stateRoot, blockHash),
            proof,
            journal
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
    }

    function testNonEmptyProof() public {
        uint256 blockNumber = 16;
        bytes32 stateRoot = 0x01eabe125b5f4f9ce2b9c3cc3c306fe789bd6f6ef28aa8d2fb2254e1be045e38;
        bytes32 blockHash = 0x01eabe125b5f4f9ce2b9c3cc3c306fe789bd6f6ef28aa8d2fb2254e1be045e38;
        bytes32 appid = 0xa40fb80ad4287819ecda5efac01c74c78d7cb00ca5f9eb5f6c0f19bd09936ac1;

        proofManager.updateNexusBlock(
            blockNumber,
            NexusProofManager.NexusBlock(stateRoot, blockHash),
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
