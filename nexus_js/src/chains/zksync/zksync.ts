import { Provider } from "zksync-ethers";
import { ZKSYNC_CHAIN_ID } from "../../constants.js";
import { ChainDetails } from "../../types.js";
import { MailboxMessageStruct } from "../../types/Mailbox.js";
import ChainInterface from "../interface.js";
import { RpcProof, StorageProofProvider } from "./storageManager.js";
import MailBoxClient from "../../mailbox.js";
import { AbiCoder, ethers } from "ethers";
import logger from "../../logger.js";
import { InterfaceAbi } from "ethers";
import { TransactionReceipt } from "ethers";

type Proof = {
  batchNumber: number;
  account: string;
  key: string;
  value: string;
  path: string[];
  index: number;
};

type ReceiveMessageArgs = {
  storageKey: string
};

export default class ZKSyncVerifier extends ChainInterface<ReceiveMessageArgs> {
  private mailboxClient: MailBoxClient;
  constructor(
    private chains: { [appId: string]: ChainDetails },
    private verifierChain: ChainDetails,
    mailboxAbi: InterfaceAbi
  ) {
    super()

    //TODO: remove this logic from constructor.
    this.mailboxClient = new MailBoxClient(
      chains,
      mailboxAbi
    );
  }

  async sendMessage(chainIdTo: string[], to: string[], nonce: number, data: string) {
    await this.mailboxClient.sendMessage(this.verifierChain.appID, chainIdTo, to, nonce, data);
  }

  async receiveMessage(
    chainblockNumber: number,
    receipt: MailboxMessageStruct,
    args: ReceiveMessageArgs
  ): Promise<TransactionReceipt> {
    const proof = await this.getStorageProof(args.storageKey, chainblockNumber, receipt.nexusAppIDFrom.toString());
    if (!proof) throw new Error("Proof not found");

    const proofSC: Proof = {
      account: proof.account,
      key: proof.key,
      path: proof.path,
      value: proof.value,
      index: proof.index,
      batchNumber: chainblockNumber,
    };

    let encodedProof = AbiCoder.defaultAbiCoder().encode(
      ["uint64", "address", "uint256", "bytes32", "bytes32[]", "uint64"],
      [
        proofSC.batchNumber,
        proofSC.account,
        proofSC.key,
        proofSC.value,
        proofSC.path,
        proofSC.index,
      ]
    );

    return await this.mailboxClient.receiveMessage(
      receipt.nexusAppIDFrom.toString(),
      chainblockNumber,
      receipt,
      encodedProof,
    );
  }

  async getStorageProof(
    storageKey: string,
    batchNumber: number,
    fromAppID: string,
  ): Promise<RpcProof | undefined> {
    const storageProofManager = new StorageProofProvider(new Provider(this.chains[fromAppID].rpcUrl));

    try {
      let proof = await storageProofManager.getProof(
        this.chains[fromAppID].mailboxContract,
        storageKey,
        batchNumber
      );
      return proof;
    } catch (e) {
      logger.error(e);
      return undefined;
    }
  }

  calculateStorageKey(key: string, slotNumber: number): string {
    return ethers.keccak256(
      AbiCoder.defaultAbiCoder().encode(
        ["uint256", "uint256"],
        [key, slotNumber]
      )
    );
  }
}
