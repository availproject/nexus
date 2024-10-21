import { MailboxMessageStruct } from "../types/Mailbox.js";

export default abstract class ChainInterface<T> {

  abstract sendMessage(chainIdTo: string[], to: string[], nonce: number, data: string): Promise<void>;

  abstract receiveMessage(
    chainblockNumber: number,
    receipt: MailboxMessageStruct,
    args: T
  ): Promise<void>;
}
