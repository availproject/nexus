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

    mapping(bytes32 => bool) public messages;
    mapping(bytes32 => INexusVerifierWrapper) public verifierWrappers;
    mapping(bytes32 => MailboxMessage) public verifiedReceipts;

    bytes32 public nexusAppID;

    event CallbackFailed(address indexed to, bytes data);

    function initialize(bytes32 _nexusAppID) public initializer {
        nexusAppID = _nexusAppID;
        __Ownable_init(msg.sender);
    }

    function receiveMessage(
        uint256 chainblockNumber,
        MailboxMessage calldata receipt,
        bytes calldata proof
    ) public {
        INexusVerifierWrapper verifier = verifierWrappers[
            receipt.nexusAppIDFrom
        ];
        if (address(verifier) == address(0)) {
            revert WrapperNotAvailable();
        }

        bytes32 receiptHash = keccak256(abi.encode(receipt));

        /// @dev we check if not exists, using nexusAppID = 0 since this can is imposed by mailbox that the nexusAppID is not 0 when storing
        if (verifiedReceipts[receiptHash].nexusAppIDFrom != bytes32(0)) {
            revert StateAlreadyUpdated();
        }

        verifier.parseAndVerify(chainblockNumber, receiptHash, proof);
        verifiedReceipts[receiptHash] = receipt;

        address to = search(receipt.nexusAppIDTo, receipt.to);
        if (to != address(0)) {
            (bool success, ) = to.call(
                abi.encodeWithSignature(
                    "onNexusMessage(bytes32, address, bytes, uint256)",
                    receipt.nexusAppIDFrom,
                    receipt.from,
                    receipt.data,
                    receipt.nonce
                )
            );
            if (!success) {
                emit CallbackFailed(to, receipt.data);
            }
        }
    }

    // @dev we take nonce from the msg.sender since they manage and create deterministic receipt structures.
    function sendMessage(
        bytes32[] memory nexusAppIDTo,
        address[] memory to,
        uint256 nonce,
        bytes calldata data
    ) public {
        //TODO: Check why the address and appId length should match.
        if (nexusAppIDTo.length != to.length) {
            revert InvalidParameters();
        }

        //TODO: Check if for now nexusAppIDTo can be a single appId so quickSort and search can be skipped.
        quickSort(nexusAppIDTo, to, 0, int256(nexusAppIDTo.length - 1));

        MailboxMessage memory receipt = MailboxMessage({
            nexusAppIDFrom: nexusAppID,
            nexusAppIDTo: nexusAppIDTo,
            data: data,
            from: msg.sender,
            to: to,
            nonce: nonce
        });

        bytes32 receiptHash = keccak256(abi.encode(receipt));

        if (messages[receiptHash]) {
            revert("Message already sent");
        }

        messages[receiptHash] = true;

        emit MailboxEvent(
            nexusAppID,
            nexusAppIDTo,
            data,
            msg.sender,
            to,
            nonce,
            receiptHash
        );
    }

    function quickSort(
        bytes32[] memory nexusAppIDTo,
        address[] memory to,
        int256 left,
        int256 right
    ) internal pure {
        int256 i = left;
        int256 j = right;
        if (i == j) return;
        bytes32 pivot = nexusAppIDTo[uint256(left + (right - left) / 2)];
        while (i <= j) {
            while (nexusAppIDTo[uint256(i)] < pivot) i++;
            while (pivot < nexusAppIDTo[uint256(j)]) j--;
            if (i <= j) {
                (nexusAppIDTo[uint256(i)], nexusAppIDTo[uint256(j)]) = (
                    nexusAppIDTo[uint256(j)],
                    nexusAppIDTo[uint256(i)]
                );
                (to[uint256(i)], to[uint256(j)]) = (
                    to[uint256(j)],
                    to[uint256(i)]
                );
                i++;
                j--;
            }
        }
        if (left < j) {
            quickSort(nexusAppIDTo, to, left, j);
        }
        if (i < right) {
            quickSort(nexusAppIDTo, to, i, right);
        }
    }

    function search(
        bytes32[] memory nexusAppIDTo,
        address[] memory to
    ) internal view returns (address) {
        if (nexusAppIDTo.length == 0) {
            return (address(0));
        }

        int256 left = 0;
        int256 right = int256(nexusAppIDTo.length - 1);

        while (left <= right) {
            int256 mid = left + (right - left) / 2;

            if (nexusAppIDTo[uint256(mid)] == nexusAppID) {
                return to[uint256(mid)];
            }

            if (nexusAppIDTo[uint256(mid)] < nexusAppID) {
                left = mid + 1;
            } else {
                right = mid - 1;
            }
        }

        return (address(0));
    }

    // @dev This function can reset a verifier wrapper back to address(0)
    function addOrUpdateWrapper(
        bytes32 wrapperChainId,
        INexusVerifierWrapper wrapper
    ) public onlyOwner {
        verifierWrappers[wrapperChainId] = wrapper;
    }
}
