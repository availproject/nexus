import { ZKSYNC_CHAIN_ID } from "../../constants";
import { Receipt } from "../../types";
import ChainInterface from "../interface";

type Proof = {
  batchNumber: number;
  account: string;
  key: number;
  value: string;
  path: string[];
  index: number;
};
export default class ZKSyncVerifier extends ChainInterface {
  private rpcUrl: string;
  constructor(_rpcUrl: string) {
    super(ZKSYNC_CHAIN_ID);
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
}
