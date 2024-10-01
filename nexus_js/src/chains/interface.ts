import { Receipt } from "../types";
import MailBoxClient from "../mailbox";

export default abstract class ChainInterface {
  protected chainId: string;

  constructor(_chainId: string) {
    this.chainId = _chainId;
  }

  abstract sendMessage(chainIdTo: string[], to: string[], data: string): void;

  abstract receiveMessage(
    chainblockNumber: number,
    receipt: Receipt,
    callback: boolean,
    ...args: any[]
  ): void;
}
