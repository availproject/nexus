import { AccountApiResponse } from "./types/nexus";
import axios from "axios";
export default class NexusClient {
  constructor(private url: string, private appId: string) { };

  async getAccountState(): Promise<AccountApiResponse> {
    let response = await axios.get(this.url + "/account-hex", {
      params: {
        app_account_id: this.appId,
      },
    });

    return {
      chainStateNumber: response.data.account.height,
      info: {
        stateRoot: "0x" + response.data.nexus_header.state_root,
      },
      response: response.data,
    };
  };
}
