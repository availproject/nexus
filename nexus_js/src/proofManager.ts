import { assert, Contract, Wallet, Provider as EthersProvider } from "ethers";
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const proofManagerAbi = require("./abi/proofManager.json");
//@ts-ignore
//import proofManagerAbi from "./abi/proofManager.json" with { type: "json" };
import { Provider } from "zksync-ethers";
import { AccountState } from "./types/index.js";
import { hexlify } from "ethers";

class ProofManagerClient {
  // provider = ethers.Provider( ... mail box ....);
  private proofManager: Contract;

  constructor(address: string, rpc: string, privateKey: string) {
    const provider = new Provider(rpc);
    //TODO: Resolve below assertion
    const wallet = new Wallet(privateKey, provider as unknown as EthersProvider);

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
    // Convert fields to the expected types
    const statementDigest = "0x" + accountState.statement;
    const stateRoot = "0x" + accountState.state_root;
    const startNexusHash = "0x" + accountState.start_nexus_hash;

    const lastProofHeight = accountState.last_proof_height;
    const height = accountState.height;
    const accountStateOnChain = {
      statementDigest,
      stateRoot,
      startNexusHash,
      lastProofHeight,
      height,
    };

    // Call the updateChainState function on the smart contract
    const response = await this.proofManager.updateChainState(
      blockNumber,
      siblings.map((value) => "0x" + value),
      "0x" + nexusAppID, // Convert the nexusAppID if needed
      accountStateOnChain // Pass the formatted account state
    );

    return response;
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
