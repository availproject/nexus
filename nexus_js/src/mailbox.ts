import { ethers, InterfaceAbi, Provider } from "ethers";
import logger from "./logger.js";
import { ChainDetails, Networks } from "./types.js";
import { MailboxMessageStruct } from "./types/Mailbox.js";
import { TransactionReceipt } from "ethers";

class MailboxUtils {
  protected chains: Map<string, ChainDetails>;

  constructor(chains: { [appId: string]: ChainDetails }, private mailboxAbi: InterfaceAbi) {
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
    nexusAppIDFrom: string,
    nexusAppIDTo: string[],
    data: string,
    from: string,
    to: string[],
    nonce: number
  ): MailboxMessageStruct | undefined {
    const chainId = this.chains.get(nexusAppIDFrom)?.chainId;
    if (!chainId) {
      return;
    }
    const receipt: MailboxMessageStruct = {
      nexusAppIDFrom: ethers.encodeBytes32String(nexusAppIDFrom),
      nexusAppIDTo: nexusAppIDTo.map((appId) => ethers.encodeBytes32String(appId)),
      data: data,
      from: ethers.encodeBytes32String(from),
      to: to.map((toAddress) => ethers.encodeBytes32String(toAddress)),
      nonce: nonce,
    };

    return receipt;
  }

  protected getMailboxContract(nexusAppId: string): ethers.Contract {
    const chainInfo = this.chains.get(nexusAppId);
    if (!chainInfo) {

      throw new Error("Chain info not known to mailbox");
    }
    const provider = new ethers.JsonRpcProvider(chainInfo.rpcUrl);

    if (!chainInfo.mailboxContract) {
      logger.error("Mailbox Contract address missing");

      throw new Error("Mailbox Contract address missing");
    }

    const mailboxContract = new ethers.Contract(
      chainInfo.mailboxContract,
      this.mailboxAbi,
      new ethers.Wallet(chainInfo.privateKey, provider)
    )

    return mailboxContract;
  }
}

export default class MailBoxClient extends MailboxUtils {
  constructor(chains: { [appId: string]: ChainDetails }, mailboxAbi: InterfaceAbi) {
    if (Object.keys(chains).length === 0) {
      throw new Error("At least one account state mapping must be provided.");
    }

    super(chains, mailboxAbi);
  }

  async sendMessage(
    nexusAppIDFrom: string,
    nexusAppIDTo: string[],
    to: string[],
    nonce: number,
    data: string
  ) {
    const mailboxContract = this.getMailboxContract(nexusAppIDFrom);
    await mailboxContract?.sendMessage(nexusAppIDTo, to, nonce, data);
  }

  async receiveMessage(
    nexusAppIDFrom: string,
    chainblockNumber: number,
    receipt: MailboxMessageStruct,
    proof: string
  ): Promise<TransactionReceipt> {
    const mailboxContract = this.getMailboxContract(nexusAppIDFrom);
    const tx = await mailboxContract?.receiveMessage(chainblockNumber, receipt, proof);

    const txReceipt = await tx.wait();

    return txReceipt;
  }

  addChain(key: string, chainDetails: ChainDetails): void {
    this.chains.set(key, chainDetails);
  }
}
