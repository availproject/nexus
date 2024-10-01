import { Provider } from "zksync-ethers";
import { ZKSYNC_CHAIN_ID } from "../../constants";
import { ChainDetails, Chains, Receipt } from "../../types";
import ChainInterface from "../interface";
import { RpcProof, StorageProofProvider } from "./storageManager";
import MailBoxClient from "../../mailbox";
import { AbiCoder, ethers } from "ethers";
import logger from "../../logger";

type Proof = {
  batchNumber: number;
  account: string;
  key: string;
  value: string;
  path: string[];
  index: number;
};

export default class ZKSyncVerifier extends ChainInterface {
  private chainDetails: ChainDetails;
  private mailbox: MailBoxClient;
  private provider: Provider;
  constructor(
    _mailbox: MailBoxClient,
    _chainDetails: ChainDetails,
    _provider: Provider
  ) {
    super(ZKSYNC_CHAIN_ID);
    this.chainDetails = _chainDetails;
    this.provider = _provider;
    this.mailbox = _mailbox;
  }

  async sendMessage(chainIdTo: string[], to: string[], data: string) {
    await this.mailbox.sendMessage(Chains.ZKSync, chainIdTo, to, data);
  }
  async receiveMessage(
    chainblockNumber: number,
    receipt: Receipt,
    callback: boolean,
    storageKey: string
  ) {
    const proof = await this.getStorageProof(storageKey, chainblockNumber);
    if (!proof) return undefined;
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

    await this.mailbox.receiveMessage(
      Chains.ZKSync,
      chainblockNumber,
      receipt,
      encodedProof,
      callback
    );
  }

  async getStorageProof(
    storageKey: string,
    batchNumber: number
  ): Promise<RpcProof | undefined> {
    const storageProofManager = new StorageProofProvider(this.provider);
    try {
      let proof = await storageProofManager.getProof(
        this.chainDetails.mailboxContract,
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
