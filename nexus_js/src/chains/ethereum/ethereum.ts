import { ETHEREUM_CHAIN_ID } from "../constants";
import { Receipt } from "../types";
import ChainInterface from "./interface";

type Proof = {
  accountProof: string;
  addr: string;
  storageProof: string;
  storageSlot: string;
};
export default class EthereumVerifier extends ChainInterface {
  private rpcUrl: string;
  constructor(_rpcUrl: string) {
    super(ETHEREUM_CHAIN_ID);
    this.rpcUrl = _rpcUrl;
  }

  sendMessage(chainIdTo: string[], to: string[]): void {}
  receiveMessage(
    chainblockNumber: number,
    receipt: Receipt,
    callback: boolean
  ): string {
    return "";
  }

  encodeData(...args: any[]): void {}

  getStorageProof() {}
  calculateStorageSlot() {}

  // move the encoding logic from rust off-chain scripts here
  private concatenate() {}
  private rlpEncode() {}
}
