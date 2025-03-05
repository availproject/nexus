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

    bytes32 private constant EMPTY_TRIE_ROOT_HASH = 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421;
    bytes32 private constant EMPTY_CODE_HASH = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470;

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
        proofManager = new NexusProofManager(address(risc0Router));
        verifier = new EthereumVerifier(INexusProofManager(address(proofManager)));
    }

    function testEmptyProof() public {
        uint256 blockNumber = 123;
        bytes32 appid = 0x3655ca59b7d566ae06297c200f98d04da2e8e89812d627bc29297c25db60362d;

        proofManager.updateNexusBlock(blockNumber, proof, journal);
        bytes32[] memory siblings;

        /*
        from account proof from nexus client :
        1. Run mock-geth adapter with nexus
        2. Fetch the account state and then send the proof
        3. Print the result.
        4. Extract the values from the result and use it here.
        5. Below info is from account state

        0xa01ae135624bda83c2804af67e0931b7069bcf5d9c68a90833460bd17acc6dd5 ---> statement digest
        0x0000000000000000000000000000000000000000000000000000000000000000 ---> state root
        0x30c23598430f6c4eb3d583a394240b281936dfc243e2417b4e8c9017a9679c56 ---> start nexus hash
        0x0000000000000000000000000000000000000000000000000000000000000002 ---> last proof height
        0x0000000000000000000000000000000000000000000000000000000000348371 ---> height
        */

        NexusProofManager.AccountState memory state = NexusProofManager.AccountState(
            0xa01ae135624bda83c2804af67e0931b7069bcf5d9c68a90833460bd17acc6dd5,
            0x0000000000000000000000000000000000000000000000000000000000000000,
            0x30c23598430f6c4eb3d583a394240b281936dfc243e2417b4e8c9017a9679c56,
            2,
            3441521
        );

        proofManager.updateChainState(blockNumber, siblings, appid, state);
    }

    function testNonEmptyProof() public {
        uint256 blockNumber = 123;
        bytes32 appid = 0x3655ca59b7d566ae06297c200f98d04da2e8e89812d627bc29297c25db60362d;

        proofManager.updateNexusBlock(blockNumber, proof, journal);
        bytes32[] memory siblings;
        NexusProofManager.AccountState memory state = NexusProofManager.AccountState(
            0xa01ae135624bda83c2804af67e0931b7069bcf5d9c68a90833460bd17acc6dd5,
            0x0000000000000000000000000000000000000000000000000000000000000000,
            0x30c23598430f6c4eb3d583a394240b281936dfc243e2417b4e8c9017a9679c56,
            2,
            3441521
        );

        proofManager.updateChainState(blockNumber, siblings, appid, state);
    }
}
