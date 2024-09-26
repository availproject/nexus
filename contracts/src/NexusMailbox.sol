// SPDX-License-Identifier: Apache 2.0
pragma solidity ^0.8.21;

import {Receipt, INexusMailbox} from "./interfaces/INexusMailbox.sol";
import {INexusVerifierWrapper} from "./interfaces/INexusVerifierWrapper.sol";
import {INexusReceiver} from "./interfaces/INexusReceiver.sol";
import {Initializable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/access/OwnableUpgradeable.sol";

contract NexusMailbox is INexusMailbox, Initializable, OwnableUpgradeable {
    error WrapperNotAvailable();
    error InvalidParameters();
    error StateAlreadyUpdated();

    mapping(bytes32 => bytes32) public sendMessages;
    mapping(bytes32 => INexusVerifierWrapper) public verifierWrappers;
    mapping(bytes32 => Receipt) public verifiedReceipts;

    bytes32 public chainId;
    uint256 public mailboxNonce;

    event CallbackFailed(address indexed to, bytes data);

    function initialise() public initializer {
        chainId = bytes32(block.chainid);
        __Ownable_init(msg.sender);
    }

    function receiveMessage(uint256 chainblockNumber, Receipt calldata receipt, bytes calldata proof, bool callback)
        public
    {
        INexusVerifierWrapper verifier = verifierWrappers[receipt.chainIdFrom];
        if (address(verifier) == address(0)) {
            revert WrapperNotAvailable();
        }

        bytes32 receiptHash = keccak256(abi.encode(receipt));

        /// @dev we check if not exists, using chainId = 0 since this can is imposed by mailbox that the chainID is not 0 when storing
        if (verifiedReceipts[keccak256(abi.encode(receipt.chainIdFrom, receiptHash))].chainIdFrom != bytes32(0)) {
            revert StateAlreadyUpdated();
        }

        verifier.parseAndVerify(chainblockNumber, receiptHash, proof);
        verifiedReceipts[keccak256(abi.encode(receipt.chainIdFrom, receiptHash))] = receipt;

        if (callback) {
            address to = search(receipt.chainIdTo, receipt.to);
            if(to != address(0)) {
                (bool success,) = to.call(
                    abi.encodeWithSignature("callback(bytes)", receipt.data)
                );
                if (!success) { 
                    emit CallbackFailed(to, receipt.data);
                }
            }
        }
    }

    function sendMessage(
        bytes32[] memory chainIdTo,
        address[] memory to,
        bytes calldata data
    ) public {
        if (chainIdTo.length != to.length) {
            revert InvalidParameters();
        }
        quickSort(chainIdTo, to,  0, int(chainIdTo.length-1));
        Receipt memory receipt = Receipt({
            chainIdFrom: chainId,
            chainIdTo: chainIdTo,
            data: data,
            from: msg.sender,
            to: to,
            nonce: mailboxNonce++
        });
        bytes32 receiptHash = keccak256(abi.encode(receipt));
        bytes32 key = keccak256(abi.encode(msg.sender, receiptHash));
        sendMessages[key] = receiptHash;
    }

    function quickSort(bytes32[] memory chainIdTo, address[] memory to,  int256 left, int256 right) internal pure {
        int256 i = left;
        int256 j = right;
        if (i == j) return;
        bytes32 pivot = chainIdTo[uint256(left + (right - left) / 2)];
        while (i <= j) {
            while (chainIdTo[uint256(i)] < pivot) i++;
            while (pivot < chainIdTo[uint256(j)]) j--;
            if (i <= j) {
                (chainIdTo[uint256(i)], chainIdTo[uint256(j)]) = (chainIdTo[uint256(j)], chainIdTo[uint256(i)]);
                (to[uint256(i)], to[uint256(j)]) = (to[uint256(j)], to[uint256(i)]);
                i++;
                j--;
            }
        }
        if (left < j) {
            quickSort(chainIdTo,to, left, j);
        }
        if (i < right) {
            quickSort(chainIdTo, to, i, right);
        }
    }

      function search(bytes32[] memory chainIdTo, address[] memory to) internal view returns (address) {
        if (chainIdTo.length == 0) {
            return (address(0));
        }
        
        int left = 0;
        int right = int(chainIdTo.length - 1);
        
        while (left <= right) {
            int mid = left + (right - left) / 2;
            
            if (chainIdTo[uint(mid)] == chainId) {
                return to[uint(mid)];
            }
            
            if (chainIdTo[uint(mid)] < chainId) {
                left = mid + 1;
            } else {
                right = mid - 1;
            }
        }
        
        return (address(0));
    }

    // @dev This function can reset a verifier wrapper back to address(0)
    function addOrUpdateWrapper(bytes32 wrapperChainId, INexusVerifierWrapper wrapper) public onlyOwner {
        verifierWrappers[wrapperChainId] = wrapper;
    }
}
