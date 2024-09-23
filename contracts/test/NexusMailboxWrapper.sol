// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;


import {NexusMailbox} from "../src/NexusMailbox.sol";
import {INexusVerifierWrapper} from "../src/interfaces/INexusVerifierWrapper.sol";
import {Receipt} from "../src/interfaces/INexusMailbox.sol";


contract NexusMailboxWrapper is NexusMailbox { 
    function updateSendMessages(uint256 key, bytes32 value) public { 
        sendMessages[bytes32(key)] = value;
    }

    function checkVerificationOfEncoding(uint256 chainblockNumber, Receipt memory receipt, bytes32 from, bytes32  receiptHash, bytes calldata proof) public {
        INexusVerifierWrapper verifier = verifierWrappers[from];
        verifier.parseAndVerify(chainblockNumber, receiptHash, proof);
        verifiedReceipts[keccak256(abi.encode(from, receiptHash))] = receipt;
    }
}