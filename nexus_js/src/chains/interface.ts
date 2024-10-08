import { MailboxMessageStruct } from "../types/Mailbox";

export default abstract class ChainInterface<T> {
  protected chainId: string;

  constructor(_chainId: string) {
    this.chainId = _chainId;
  }

  abstract sendMessage(chainIdTo: string[], to: string[], nonce: number, data: string): Promise<void>;

  abstract receiveMessage(
    chainblockNumber: number,
    receipt: MailboxMessageStruct,
    args: T
  ): Promise<void>;
}
