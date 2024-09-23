// SPDX-License-Identifier: Apache 2.0
pragma solidity ^0.8.21;

import {Receipt, INexusMailbox} from "./interfaces/INexusMailbox.sol";
import {INexusVerifierWrapper} from "./interfaces/INexusVerifierWrapper.sol";
contract Mailbox is INexusMailbox { 
    error WrapperNotAvailable();
    error InvalidParameters();
    error StateAlreadyUpdated();

    mapping(bytes32 => bytes32) public sendMessages;
    mapping(bytes32 => INexusVerifierWrapper) public verifierWrappers;
    mapping(bytes32 => Receipt) public verifiedReceipts;

    bytes32 chainId;
    uint256 mailboxNonce;

    function initialise() public {
        chainId = bytes32(block.chainid);
    }
    function receiveMessage(uint256 chainblockNumber, Receipt calldata receipt, bytes calldata proof, bool callback) public {
        INexusVerifierWrapper verifier = verifierWrappers[receipt.chainIdFrom];
        if(address(verifier) == address(0)) {
            revert WrapperNotAvailable();
        }

        bytes32 receiptHash = keccak256(abi.encode(receipt));

        /// @dev we check if not exists, using chainId = 0 since this can is imposed by mailbox that the chainID is not 0 when storing
        if(verifiedReceipts[keccak256(abi.encode(receipt.chainIdFrom, receiptHash))].chainIdFrom != bytes32(0)) {
            revert StateAlreadyUpdated();
        }

        verifier.parseAndVerify(chainblockNumber, receiptHash, proof);
        verifiedReceipts[keccak256(abi.encode(receipt.chainIdFrom, receiptHash))] = receipt;

        if(callback) {
            // do something
        }
    }

    function sendMessage(bytes32[] calldata chainIdTo, address[] calldata to, address[] calldata  sm, bytes calldata data) public {
        if(chainIdTo.length != to.length || to.length != sm.length) {
            revert InvalidParameters();
        }
        Receipt memory receipt = Receipt ({
            chainIdFrom: chainId,
            chainIdTo: chainIdTo,
            data: data,
            from: msg.sender,
            to: to,
            sm: sm,
            nonce: mailboxNonce++
        });
        bytes32 receiptHash = keccak256(abi.encode(receipt));
        bytes32 key = keccak256(abi.encode(msg.sender, receiptHash));
        sendMessages[key] = receiptHash;
    }  
    

    // TODO: make only admin
    // @dev This function can reset a verifier wrapper back to address(0)
    function addOrUpdateWrapper(bytes32 wrapperChainId, INexusVerifierWrapper wrapper) public { 
        verifierWrappers[wrapperChainId] = wrapper;
    }
}