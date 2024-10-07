import { Mailbox, Mailbox__factory } from "./types/index";
import { ethers, Provider } from "ethers";
import logger from "./logger";
import { ChainDetails, Networks } from "./types";
import { MailboxMessageStruct } from "./types/Mailbox";

class MailboxUtils {
  protected networks: Map<string, ChainDetails>;

  constructor(networks: { [key in Networks]: ChainDetails }) {
    this.networks = new Map();
    for (let key in networks) {
      const chainKey = key as Networks;
      this.networks.set(chainKey, networks[chainKey]);
    }
  }

  public getChains(): Map<string, ChainDetails> {
    return this.networks;
  }

  public newReceipt(
    network: Networks,
    networkTo: string[],
    data: string,
    from: string,
    to: string[],
    nonce: number
  ): MailboxMessageStruct | undefined {
    const chainId = this.networks.get(network)?.chainId;
    if (!chainId) {
      return;
    }
    const receipt: MailboxMessageStruct = {
      nexusAppIdFrom: ethers.encodeBytes32String(network),
      nexusAppIdTo: networkTo,
      data: data,
      from: from,
      to: to,
      nonce: nonce,
    };
    return receipt;
  }

  protected getMailboxContract(chain: Networks): Mailbox | undefined {
    const chainInfo = this.networks.get(chain);
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
  constructor(chains: { [key in Networks]: ChainDetails }) {
    if (Object.keys(chains).length === 0) {
      throw new Error("At least one account state mapping must be provided.");
    }

    super(chains);
  }

  sendMessage(
    chain: Networks,
    chainIdTo: string[],
    to: string[],
    nonce: number,
    data: string
  ) {
    const mailboxContract = this.getMailboxContract(chain);
    mailboxContract?.sendMessage(chainIdTo, to, nonce, data);
  }

  receiveMessage(
    chain: Networks,
    chainblockNumber: number,
    receipt: MailboxMessageStruct,
    proof: string
  ) {
    const mailboxContract = this.getMailboxContract(chain);
    mailboxContract?.receiveMessage(chainblockNumber, receipt, proof);
  }

  addChain(key: string, chainDetails: ChainDetails): void {
    this.networks.set(key, chainDetails);
  }
}
