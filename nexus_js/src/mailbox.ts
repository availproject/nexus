enum Chains {
  Ethereum,
  ZKSync,
}
interface ChainDetails {
  rpcUrl: string;
  mailboxContract: string;
  stateManagerContract: string;
  appID: string; // nexus app id ?
}

export default class MailBoxClient {
  private chains: Map<string, ChainDetails>;

  constructor(chains: { [key: string]: ChainDetails }) {
    // Ensure that at least one mapping is provided during initialization
    if (Object.keys(chains).length === 0) {
      throw new Error("At least one account state mapping must be provided.");
    }

    this.chains = new Map();
    for (let key in chains) {
      if (chains.hasOwnProperty(key)) {
        this.chains.set(key, chains[key]);
      }
    }
  }

  verify(chain: Chains) {
    switch (chain) {
      case Chains.Ethereum: {
        // call ethereum verifier object using the above chains array details
      }
      case Chains.ZKSync: {
      }
      default: {
        // handle error gracefully
      }
    }
  }

  addChain(key: string, chainDetails: ChainDetails): void {
    this.chains.set(key, chainDetails);
  }
}
