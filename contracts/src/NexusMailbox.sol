// SPDX-License-Identifier: Apache 2.0
pragma solidity ^0.8.21;

import {MailboxMessage, INexusMailbox} from "./interfaces/INexusMailbox.sol";
import {INexusVerifierWrapper} from "./interfaces/INexusVerifierWrapper.sol";
import {INexusReceiver} from "./interfaces/INexusReceiver.sol";
import {Initializable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/access/OwnableUpgradeable.sol";

contract NexusMailbox is INexusMailbox, Initializable, OwnableUpgradeable {
    error WrapperNotAvailable();
    error InvalidParameters();
    error StateAlreadyUpdated();

    mapping(bytes32 => bytes32) public messages;
    mapping(bytes32 => INexusVerifierWrapper) public verifierWrappers;
    mapping(bytes32 => MailboxMessage) public verifiedReceipts;

    bytes32 public nexusAppId;

    event CallbackFailed(address indexed to, bytes data);

    function initialize(bytes32 _nexusAppId) public initializer {
        nexusAppId = _nexusAppId;
        __Ownable_init(msg.sender);
    }

    function receiveMessage(uint256 chainblockNumber, MailboxMessage calldata receipt, bytes calldata proof) public {
        INexusVerifierWrapper verifier = verifierWrappers[receipt.nexusAppIdFrom];
        if (address(verifier) == address(0)) {
            revert WrapperNotAvailable();
        }

        bytes32 receiptHash = keccak256(abi.encode(receipt));
        bytes32 key = keccak256(abi.encode(receipt.nexusAppIdFrom, receiptHash));

        /// @dev we check if not exists, using nexusAppId = 0 since this can is imposed by mailbox that the nexusAppId is not 0 when storing
        if (verifiedReceipts[key].nexusAppIdFrom != bytes32(0)) {
            revert StateAlreadyUpdated();
        }

        verifier.parseAndVerify(chainblockNumber, receiptHash, proof);
        verifiedReceipts[key] = receipt;

        address to = search(receipt.nexusAppIdTo, receipt.to);
        if (to != address(0)) {
            (bool success,) = to.call(
                abi.encodeWithSignature(
                    "onNexusMessage(bytes32, address, bytes)", receipt.nexusAppIdFrom, receipt.from, receipt.data
                )
            );
            if (!success) {
                emit CallbackFailed(to, receipt.data);
            }
        }
    }

    // @dev we take nonce from the msg.sender since they manage and create deterministic receipt structures.
    function sendMessage(bytes32[] memory nexusAppIdTo, address[] memory to, uint256 nonce, bytes calldata data)
        public
    {
        if (nexusAppIdTo.length != to.length) {
            revert InvalidParameters();
        }
        quickSort(nexusAppIdTo, to, 0, int256(nexusAppIdTo.length - 1));
        MailboxMessage memory receipt = MailboxMessage({
            nexusAppIdFrom: nexusAppId,
            nexusAppIdTo: nexusAppIdTo,
            data: data,
            from: msg.sender,
            to: to,
            nonce: nonce
        });
        bytes32 receiptHash = keccak256(abi.encode(receipt));
        bytes32 key = keccak256(abi.encode(msg.sender, receiptHash));
        messages[key] = receiptHash;
        emit MailboxEvent(nexusAppId, nexusAppIdTo, data, msg.sender, to, nonce);
    }

    function quickSort(bytes32[] memory nexusAppIdTo, address[] memory to, int256 left, int256 right) internal pure {
        int256 i = left;
        int256 j = right;
        if (i == j) return;
        bytes32 pivot = nexusAppIdTo[uint256(left + (right - left) / 2)];
        while (i <= j) {
            while (nexusAppIdTo[uint256(i)] < pivot) i++;
            while (pivot < nexusAppIdTo[uint256(j)]) j--;
            if (i <= j) {
                (nexusAppIdTo[uint256(i)], nexusAppIdTo[uint256(j)]) =
                    (nexusAppIdTo[uint256(j)], nexusAppIdTo[uint256(i)]);
                (to[uint256(i)], to[uint256(j)]) = (to[uint256(j)], to[uint256(i)]);
                i++;
                j--;
            }
        }
        if (left < j) {
            quickSort(nexusAppIdTo, to, left, j);
        }
        if (i < right) {
            quickSort(nexusAppIdTo, to, i, right);
        }
    }

    function search(bytes32[] memory nexusAppIdTo, address[] memory to) internal view returns (address) {
        if (nexusAppIdTo.length == 0) {
            return (address(0));
        }

        int256 left = 0;
        int256 right = int256(nexusAppIdTo.length - 1);

        while (left <= right) {
            int256 mid = left + (right - left) / 2;

            if (nexusAppIdTo[uint256(mid)] == nexusAppId) {
                return to[uint256(mid)];
            }

            if (nexusAppIdTo[uint256(mid)] < nexusAppId) {
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
