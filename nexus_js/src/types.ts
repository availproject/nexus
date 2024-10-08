export enum Networks {
  Ethereum = "Ethereum",
  ZKSync = "ZKSync",
}
export type ChainDetails = {
  rpcUrl: string;
  mailboxContract: string;
  stateManagerContract: string;
  appID: string; // nexus app id ?,
  chainId: string;
  type: Networks,
};
