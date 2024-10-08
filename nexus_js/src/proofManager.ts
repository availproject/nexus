import { Contract, Wallet } from "ethers";
import proofManagerAbi from "./abi/proofManager.json";
import { Provider } from "zksync-ethers";
import { AccountState } from "./types/index";

class ProofManagerClient {
  // provider = ethers.Provider( ... mail box ....);
  private proofManager: Contract;

  constructor(address: string, rpc: string, privateKey: string) {
    const provider = new Provider(rpc);
    const wallet = new Wallet(privateKey, provider);

    // can make this modular and have a mapping between chain ids and mailbox. Imo not necessary since MailBoxClient already maintains it.
    this.proofManager = new Contract(address, proofManagerAbi, wallet);
  }

  async updateNexusBlock(
    blockNumber: number,
    stateHash: string,
    blockHash: string,
    proof: string
  ) {
    const response = await this.proofManager.updateNexusBlock(blockNumber, {
      stateRoot: stateHash,
      blockHash,
    });
  }

  async updateChainState(
    blockNumber: number,
    siblings: string[],
    nexusAppID: string,
    accountState: AccountState
  ) {
    const response = await this.proofManager.updateChainState(
      blockNumber,
      siblings,
      "0x" + nexusAppID,
      accountState
    );
  }

  async getChainState(
    nexusAppID: string,
    blockNumber: number = 0
  ): Promise<string> {
    const response = await this.proofManager.getChainState(
      blockNumber,
      nexusAppID
    );

    return response;
  }
}

export { ProofManagerClient };
