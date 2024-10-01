import { Receipt } from "../types";

export default abstract class ChainInterface {
  protected chainId: string;

  constructor(_chainId: string) {
    this.chainId = _chainId;
  }

  abstract sendMessage(chainIdTo: string[], to: string[]): void;
  abstract receiveMessage(
    chainblockNumber: number,
    receipt: Receipt,
    callback: boolean
  ): string;

  abstract encodeData(...args: any[]): void;
}
