type NexusState = {
  stateRoot: string;
};

type AccountState = {
  statement: string;
  state_root: string;
  start_nexus_hash: string;
  last_proof_height: number;
  height: number;
};

type AccountApiResponse = {
  info: NexusState;
  chainStateNumber: number;
  response: {
    account: AccountState;
    proof: string[];
    value_hash: string;
    nexus_header: {
      parent_hash: string;
      prev_state_root: string;
      state_root: string;
      avail_header_hash: string;
      number: number;
    };
    value_hash_hex: string;
  };
};

export {
  NexusState,
  AccountState,
  AccountApiResponse,
}
