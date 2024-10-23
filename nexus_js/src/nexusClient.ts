import { AccountApiResponse } from "./types/nexus.js";
import axios from "axios";
class NexusClient {
  constructor(private url: string, private appId: string) { }

  async getAccountState(): Promise<AccountApiResponse> {
    let response = await axios.get(this.url + "/account-hex", {
      params: {
        app_account_id: this.remove0xPrefix(this.appId),
      },
    });

    console.log(response);

    return {
      chainStateNumber: response.data.account.height,
      info: {
        stateRoot: "0x" + response.data.nexus_header.state_root,
      },
      response: response.data,
    };
  }

  private remove0xPrefix(str: string): string {
    if (str.startsWith("0x")) {
      return str.slice(2); // Remove the first two characters (0x)
    }
    return str;
  }
}

export { NexusClient };
