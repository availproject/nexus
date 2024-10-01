type AccountState = {
  statementDigest: string;
  tateRoot: string;
  startNexusHash: string;
  lastProofHeight: number;
  height: number;
};

export default class MailBoxClient {
  // provider = ethers.Provider( ... mail box ....);

  constructor(address: string, rpc: string) {
    // can make this modular and have a mapping between chain ids and mailbox. Imo not necessary since MailBoxClient already maintains it.
  }

  updateNexusBlock(
    blocknumber: number,
    stateHash: string,
    blockHash: string,
    proof: string
  ) {
    // calls update nexus block on state manager
  }

  updateChainState(
    blocknumber: number,
    siblings: string[],
    key: string,
    accountState: AccountState
  ) {
    // update the state of a particular chain
  }

  getChainState(nexusAppID: string, blocknumber?: number) {
    //get the nexus state. If block number not provided, take it as 0, SC returns the latest known in that case
  }
}
