// SPDX-License-Identifier: Apache 2.0
pragma solidity ^0.8.21;

import {MailboxMessage, INexusMailbox, VerifierInfo} from "./interfaces/INexusMailbox.sol";
import {INexusVerifierWrapper} from "./interfaces/INexusVerifierWrapper.sol";
import {INexusReceiver} from "./interfaces/INexusReceiver.sol";
import {Initializable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import {OwnableUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/access/OwnableUpgradeable.sol";

/// @title NexusMailbox
/// @author Rachit Anand Srivastava (@privacy_prophet)
/// @notice Handles cross-chain message passing between different Nexus applications
/// @dev Implements message verification and delivery using verifier wrappers
contract NexusMailbox is INexusMailbox, Initializable, OwnableUpgradeable {
    error WrapperNotAvailable();
    error InvalidParameters();
    error StateAlreadyUpdated();

    /// @notice Tracks whether a message hash has been processed
    mapping(bytes32 => bool) public messages;
    /// @notice Maps chain IDs to their verifier wrapper contracts
    mapping(bytes32 => VerifierInfo) public verifierWrappers;
    /// @notice Stores verified message receipts by their hash
    mapping(bytes32 => MailboxMessage) public verifiedMessages;
    /// @notice Stores sent message details by their hash
    mapping(bytes32 => MailboxMessage) private sendMessages;
    /// @notice The unique identifier for this Nexus application
    bytes32 public nexusAppID;

    /// @notice Emitted when a message callback fails
    /// @param to The target address that failed to process the message
    /// @param data The message data that failed to process
    event CallbackFailed(address indexed to, bytes data);

    /// @notice Initializes the contract with a Nexus application ID
    /// @param _nexusAppID The unique identifier for this Nexus application
    function initialize(bytes32 _nexusAppID) public initializer {
        nexusAppID = _nexusAppID;
        __Ownable_init(msg.sender);
    }

    /// @notice Receives and processes a cross-chain message
    /// @param chainblockNumber The block number from the source chain
    /// @param receipt The message receipt containing all message details
    /// @param proof The proof verifying the authenticity of the message
    function receiveMessage(
        uint256 chainblockNumber,
        MailboxMessage calldata receipt,
        bytes calldata proof
    ) public {
        VerifierInfo memory verifierInfo = verifierWrappers[
            receipt.nexusAppIDFrom
        ];
        if (address(verifierInfo.verifier) == address(0)) {
            revert WrapperNotAvailable();
        }

        bytes32 receiptHash = keccak256(abi.encode(receipt));

        /// @dev we check if not exists, using nexusAppID = 0 since this can is imposed by mailbox that the nexusAppID is not 0 when storing
        if (verifiedMessages[receiptHash].nexusAppIDFrom != bytes32(0)) {
            revert StateAlreadyUpdated();
        }

        verifierInfo.verifier.parseAndVerify(
            chainblockNumber,
            receiptHash,
            proof,
            verifierInfo.mailboxAddress
        );
        verifiedMessages[receiptHash] = receipt;

        address to = search(receipt.nexusAppIDTo, receipt.to);

        if (to != address(0)) {
            (bool success, ) = to.call(
                abi.encodeWithSignature(
                    "onNexusMessage(bytes32,address,bytes,uint256)",
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

    /// @notice Sends a cross-chain message to one or more destinations
    /// @dev We take nonce from the msg.sender since they manage and create deterministic receipt structures
    /// @param nexusAppIDTo Array of destination Nexus application IDs
    /// @param to Array of destination addresses
    /// @param nonce The message nonce
    /// @param data The message payload
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
        sendMessages[receiptHash] = receipt;

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

    /// @notice Retrieves the details of a sent message
    /// @param receiptHash The hash of the message receipt
    /// @return The full message details
    function getSendMessage(
        bytes32 receiptHash
    ) public view returns (MailboxMessage memory) {
        return sendMessages[receiptHash];
    }

    /// @notice Retrieves a verified message receipt
    /// @param receiptHash The hash of the message receipt
    /// @return The verified message details
    function getVerifiedMessage(
        bytes32 receiptHash
    ) public view returns (MailboxMessage memory) {
        return verifiedMessages[receiptHash];
    }

    /// @notice Sorts arrays of Nexus application IDs and addresses in parallel
    /// @dev Uses quicksort algorithm to sort both arrays based on nexusAppIDTo values
    /// @param nexusAppIDTo Array of Nexus application IDs to sort
    /// @param to Array of addresses to sort in parallel
    /// @param left The leftmost index of the sort range
    /// @param right The rightmost index of the sort range
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

    /// @notice Searches for a matching Nexus application ID and returns its associated address
    /// @dev Uses binary search on sorted arrays
    /// @param nexusAppIDTo Array of Nexus application IDs to search
    /// @param to Array of addresses corresponding to the application IDs
    /// @return The matching address or address(0) if not found
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

    /// @notice Adds or updates a verifier wrapper for a specific chain
    /// @dev This function can reset a verifier wrapper back to address(0)
    /// @param nexusAppIDFrom The chain ID to set the wrapper for
    /// @param verifierInfo The verifier wrapper contract address
    function addOrUpdateWrapper(
        bytes32 nexusAppIDFrom,
        VerifierInfo memory verifierInfo
    ) public onlyOwner {
        verifierWrappers[nexusAppIDFrom] = verifierInfo;
    }
}
