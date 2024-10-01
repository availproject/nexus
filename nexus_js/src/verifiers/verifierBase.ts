type Receipt = {
  chainIdFrom: string; // bytes32 padded
  chainIdTo: string[];
  data: string;
  from: string;
  to: string[];
  nonce: number;
};

export default abstract class Verifier {
  abstract parseAndVerify(
    receipt: Receipt,
    data: string,
    blockNumber?: number
  ): boolean;
}
