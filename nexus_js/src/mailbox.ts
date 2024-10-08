import { Mailbox, Mailbox__factory } from "./types/index";
import { ethers, Provider } from "ethers";
import logger from "./logger";
import { ChainDetails, Networks } from "./types";
import { MailboxMessageStruct } from "./types/Mailbox";

class MailboxUtils {
  protected chains: Map<string, ChainDetails>;

  constructor(chains: { [appId: string]: ChainDetails }) {
    this.chains = new Map<string, ChainDetails>();
    for (const appId in chains) {
      if (Object.prototype.hasOwnProperty.call(chains, appId)) {
        this.chains.set(appId, chains[appId]);
      }
    }
  }

  public getChains(): Map<string, ChainDetails> {
    return this.chains;
  }

  public newReceipt(
    nexusAppIdFrom: string,
    nexusAppIdTo: string[],
    data: string,
    from: string,
    to: string[],
    nonce: number
  ): MailboxMessageStruct | undefined {
    const chainId = this.chains.get(nexusAppIdFrom)?.chainId;
    if (!chainId) {
      return;
    }
    const receipt: MailboxMessageStruct = {
      nexusAppIdFrom: ethers.encodeBytes32String(nexusAppIdFrom),
      nexusAppIdTo: nexusAppIdTo.map((appId) => ethers.encodeBytes32String(appId)),
      data: data,
      from: ethers.encodeBytes32String(from),
      to: to.map((toAddress) => ethers.encodeBytes32String(toAddress)),
      nonce: nonce,
    };

    return receipt;
  }

  protected getMailboxContract(nexusAppId: string): Mailbox | undefined {
    const chainInfo = this.chains.get(nexusAppId);
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
  constructor(chains: { [appId: string]: ChainDetails }) {
    if (Object.keys(chains).length === 0) {
      throw new Error("At least one account state mapping must be provided.");
    }

    super(chains);
  }

  async sendMessage(
    nexusAppIdFrom: string,
    nexusAppIdTo: string[],
    to: string[],
    nonce: number,
    data: string
  ) {
    const mailboxContract = this.getMailboxContract(nexusAppIdFrom);
    await mailboxContract?.sendMessage(nexusAppIdTo, to, nonce, data);
  }

  async receiveMessage(
    nexusAppIdFrom: string,
    chainblockNumber: number,
    receipt: MailboxMessageStruct,
    proof: string
  ) {
    const mailboxContract = this.getMailboxContract(nexusAppIdFrom);
    await mailboxContract?.receiveMessage(chainblockNumber, receipt, proof);
  }

  addChain(key: string, chainDetails: ChainDetails): void {
    this.chains.set(key, chainDetails);
  }
}
