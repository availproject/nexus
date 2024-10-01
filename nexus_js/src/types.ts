export enum Chains {
  Ethereum = "Ethereum",
  ZKSync = "ZKSync",
}
export type ChainDetails = {
  rpcUrl: string;
  mailboxContract: string;
  stateManagerContract: string;
  appID: string; // nexus app id ?,
  chainId: string;
};

export type Receipt = {
  chainIdFrom: string;
  chainIdTo: string[];
  data: string;
  from: string;
  to: string[];
  nonce: number;
};
