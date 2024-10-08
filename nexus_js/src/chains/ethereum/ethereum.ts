// import { ETHEREUM_CHAIN_ID } from "../../constants";
// import { Receipt } from "../../types";
// import MailBoxClient from "../../mailbox";
// import ChainInterface from "../interface";

// type Proof = {
//   accountProof: string;
//   addr: string;
//   storageProof: string;
//   storageSlot: string;
// };
// export default class EthereumVerifier extends ChainInterface {
//   private rpcUrl: string;
//   private mailbox: MailBoxClient;

//   constructor(_mailbox: MailBoxClient, _rpcUrl: string) {
//     super(ETHEREUM_CHAIN_ID);
//     this.rpcUrl = _rpcUrl;
//     this.mailbox = _mailbox;
//   }

//   sendMessage(chainIdTo: string[], to: string[]) {
//     const storageSlot = this.calculateStorageSlot();
//     const proof = this.getProof();
//     // TODO: next steps
//   }
//   receiveMessage(
//     chainblockNumber: number,
//     receipt: Receipt,
//     callback: boolean
//   ) {}

//   getProof() {}
//   calculateStorageSlot() {}

//   // move the encoding logic from rust off-chain scripts here
//   private concatenate() {}
//   private rlpEncode() {}
// }
