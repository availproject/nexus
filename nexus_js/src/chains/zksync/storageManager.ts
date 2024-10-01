import { Provider as L2Provider } from "zksync-ethers";
import { RpcProof } from "./types";

/** Storage proof provider for zkSync */
export class StorageProofProvider {
  /**
    Estimation of difference between latest L2 batch and latest verified L1
    batch. Assuming a 30 hour delay, divided to 12 minutes per block.
  */
  readonly BLOCK_QUERY_OFFSET = 150;

  constructor(public l2Provider: L2Provider) {}

  /** Updates L2 provider */
  public setL2Provider(provider: L2Provider) {
    this.l2Provider = provider;
  }

  /** Returns ZkSync proof response */
  private async getL2Proof(
    account: string,
    storageKeys: Array<string>,
    batchNumber: number
  ): Promise<Array<RpcProof>> {
    type ZksyncProofResponse = {
      key: string;
      proof: Array<string>;
      value: string;
      index: number;
    };

    try {
      const { storageProof: storageProofs } = await this.l2Provider.send(
        "zks_getProof",
        [account, storageKeys, batchNumber]
      );

      return storageProofs.map((storageProof: ZksyncProofResponse) => {
        const { proof, ...rest } = storageProof;
        return { account, path: proof, ...rest };
      });
    } catch (e) {
      throw new Error(`Failed to get proof from L2 provider, ${e}`);
    }
  }

  /**
   * Gets the proof and related data for the given batch number, address and storage keys.
   * @param address
   * @param storageKeys
   * @param batchNumber
   * @returns
   */
  async getProofs(
    address: string,
    storageKeys: Array<string>,
    batchNumber?: number
  ): Promise<RpcProof[]> {
    // If batch number is not provided, get the latest batch number
    if (batchNumber == undefined) {
      const latestBatchNumber = await this.l2Provider.getL1BatchNumber();
      batchNumber = latestBatchNumber - this.BLOCK_QUERY_OFFSET;
    }
    const proofs = await this.getL2Proof(address, storageKeys, batchNumber);
    return proofs;
  }

  /**
   * Gets a single proof
   * @param address
   * @param storageKey
   * @param batchNumber
   * @returns
   */
  async getProof(
    address: string,
    storageKey: string,
    batchNumber?: number
  ): Promise<RpcProof> {
    const proofs = await this.getProofs(address, [storageKey], batchNumber);

    return { ...proofs[0] };
  }
}

export * from "./types";
