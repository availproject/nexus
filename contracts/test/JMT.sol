// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import "forge-std/test.sol";
import "../src/NexusProofManager.sol";
import {Structs} from "../src/lib/Structs.sol";
import "../src/interfaces/INexusProofManager.sol";
import "../src/mock/ERC20.sol";
import "../src/verification/ethereum/Verifier.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {RiscZeroCheats} from "risc0/test/RiscZeroCheats.sol";
import {ImageID} from "../src/NexusProverImageID.sol";
import {IVectorx} from "../src/interfaces/IVectorx.sol";

contract EthereumVerifierTest is Test, RiscZeroCheats {
    NexusProofManager proofManager;
    ERC20Token erc20;
    EthereumVerifier verifier;
    RiscZeroVerifierRouter risc0Router;
    IRiscZeroVerifier risc0Verifier;
    IVectorx vectorX;

    bytes32 private constant EMPTY_TRIE_ROOT_HASH = 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421;
    bytes32 private constant EMPTY_CODE_HASH = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470;

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
        risc0Router = new RiscZeroVerifierRouter(msg.sender);

        // TODO : need to change this with actual implementation when writing tests
        vectorX = IVectorx(msg.sender);

        vm.prank(msg.sender);
        risc0Router.addVerifier(bytes4(0), risc0Verifier);
        proofManager = new NexusProofManager(address(risc0Router), ImageID.NEXUS_RUNTIME_ID, address(vectorX));
        verifier = new EthereumVerifier(INexusProofManager(address(proofManager)));
    }

    function testEmptyProof() public {
        uint256 blockNumber = 15;
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

        0xb0abae3a180de2034da3ce8084bc0afcbea6a5f64efb4bb3c9b9c4a841fdaa62 ---> statement digest
        0x78960bb207f285632596f8932b98117adec87241e5c09346cbc5637ede0df7c4 ---> state root
        0xadb3f87a06a0f912592ba7c77aa784a383c80486e75969b48f8cb5453bcc91ba ---> start nexus hash
        0x0000000000000000000000000000000000000000000000000000000000000001 ---> last proof height
        0x00000000000000000000000000000000000000000000000000000000000614ea ---> height
        */

        Structs.AppState memory state = Structs.AppState(
            0xb0abae3a180de2034da3ce8084bc0afcbea6a5f64efb4bb3c9b9c4a841fdaa62,
            0x78960bb207f285632596f8932b98117adec87241e5c09346cbc5637ede0df7c4,
            0xadb3f87a06a0f912592ba7c77aa784a383c80486e75969b48f8cb5453bcc91ba,
            1,
            398570
        );

        proofManager.updateChainState(blockNumber, siblings, appid, state);
    }

    function testNonEmptyProof() public {
        uint256 blockNumber = 15;
        bytes32 appid = 0x3655ca59b7d566ae06297c200f98d04da2e8e89812d627bc29297c25db60362d;

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
    }
}
