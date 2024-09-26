// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.21;

import "../../lib/external/trie/Lib_SecureMerkleTrie.sol";
import "../../lib/external/rlp/Lib_RLPReader.sol";

contract StorageProof {
    using Lib_RLPReader for Lib_RLPReader.RLPItem;
    using Lib_RLPReader for bytes;

    uint8 private constant ACCOUNT_NONCE_INDEX = 0;
    uint8 private constant ACCOUNT_BALANCE_INDEX = 1;
    uint8 private constant ACCOUNT_STORAGE_ROOT_INDEX = 2;
    uint8 private constant ACCOUNT_CODE_HASH_INDEX = 3;

    bytes32 private constant EMPTY_TRIE_ROOT_HASH = 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421;
    bytes32 private constant EMPTY_CODE_HASH = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470;

    function verifyAccount(bytes32 latestState, bytes memory accountTrieProof, address account)
        public
        pure
        returns (uint256 nonce, uint256 accountBalance, bytes32 codeHash, bytes32 storageRoot)
    {
        bytes memory accountKey = abi.encodePacked(account);

        (bool doesAccountExist, bytes memory accountRLP) =
            Lib_SecureMerkleTrie.get(accountKey, accountTrieProof, latestState);

        (nonce, accountBalance, storageRoot, codeHash) = _decodeAccountFields(doesAccountExist, accountRLP);
    }

    function _decodeAccountFields(bool doesAccountExist, bytes memory accountRLP)
        internal
        pure
        returns (uint256 nonce, uint256 balance, bytes32 storageRoot, bytes32 codeHash)
    {
        if (!doesAccountExist) {
            return (0, 0, EMPTY_TRIE_ROOT_HASH, EMPTY_CODE_HASH);
        }

        Lib_RLPReader.RLPItem[] memory accountFields = accountRLP.toRLPItem().readList();

        nonce = accountFields[ACCOUNT_NONCE_INDEX].readUint256();
        balance = accountFields[ACCOUNT_BALANCE_INDEX].readUint256();
        codeHash = accountFields[ACCOUNT_CODE_HASH_INDEX].readBytes32();
        storageRoot = accountFields[ACCOUNT_STORAGE_ROOT_INDEX].readBytes32();
    }

    function verifyStorage(bytes32 storageRoot, bytes32 slot, bytes memory storageSlotTrieProof)
        public
        pure
        returns (bytes32 slotValue)
    {
        bytes memory storageKey = abi.encodePacked(slot);

        (, bytes memory slotValueRLP) = Lib_SecureMerkleTrie.get(storageKey, storageSlotTrieProof, storageRoot);

        slotValue = slotValueRLP.toRLPItem().readBytes32();
    }
}
