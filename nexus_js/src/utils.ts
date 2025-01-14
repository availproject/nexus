import { ethers } from "ethers";

function getStorageLocationForReceipt(receiptHash: string): string {
  const MESSAGES_MAPPING_SLOT = 0;

  const encoded = ethers.AbiCoder.defaultAbiCoder().encode(
    ["bytes32", "uint256"],
    [receiptHash, MESSAGES_MAPPING_SLOT]
  );

  return ethers.keccak256(encoded);
}
export { getStorageLocationForReceipt };
