// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";

contract ERC721Token is ERC721 {
    constructor(string memory token, string memory symbol) ERC721(token, symbol) {}

    function mint(address to, uint256 count) public {
        _mint(to, count);
    }
}
