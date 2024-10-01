import { Mailbox, Mailbox__factory } from "./types/index";
import { ethers, Provider } from "ethers";
import logger from "./logger";
import { ChainDetails, Chains, Receipt } from "./types";

class MailboxUtils {
  protected chains: Map<string, ChainDetails>;

  constructor(chains: { [key in Chains]: ChainDetails }) {
    this.chains = new Map();
    for (let key in chains) {
      const chainKey = key as Chains;
      this.chains.set(chainKey, chains[chainKey]);
    }
  }

  public getChains(): Map<string, ChainDetails> {
    return this.chains;
  }

  public newReceipt(
    chain: Chains,
    chainIdTo: string[],
    data: string,
    from: string,
    to: string[],
    nonce: number
  ): Receipt | undefined {
    const chainId = this.chains.get(chain)?.chainId;
    if (!chainId) {
      return;
    }
    const receipt: Receipt = {
      chainIdFrom: ethers.encodeBytes32String(chainId),
      chainIdTo: chainIdTo,
      data: data,
      from: from,
      to: to,
      nonce: nonce,
    };
    return receipt;
  }

  protected getMailboxContract(chain: Chains): Mailbox | undefined {
    const chainInfo = this.chains.get(chain);
    const provider = new ethers.JsonRpcProvider(chainInfo?.rpcUrl);
    if (!chainInfo?.mailboxContract) {
      logger.error("Mailbox Contract address missing");
      return;
    }
    const mailboxContract = Mailbox__factory.connect(
      chainInfo?.mailboxContract,
      provider
    );
    return mailboxContract;
  }
}

export default class MailBoxClient extends MailboxUtils {
  private: Provider;

  constructor(chains: { [key in Chains]: ChainDetails }) {
    if (Object.keys(chains).length === 0) {
      throw new Error("At least one account state mapping must be provided.");
    }

    super(chains);
  }

  sendMessage(chain: Chains, chainIdTo: string[], to: string[], data: string) {
    const mailboxContract = this.getMailboxContract(chain);
    mailboxContract?.sendMessage(chainIdTo, to, data);
  }

  receiveMessage(
    chain: Chains,
    chainblockNumber: number,
    receipt: Receipt,
    proof: string,
    callback: boolean
  ) {
    const mailboxContract = this.getMailboxContract(chain);
    mailboxContract?.receiveMessage(chainblockNumber, receipt, proof, callback);
  }

  addChain(key: string, chainDetails: ChainDetails): void {
    this.chains.set(key, chainDetails);
  }
}
