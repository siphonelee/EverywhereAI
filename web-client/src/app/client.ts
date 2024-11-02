import { createThirdwebClient } from "thirdweb";
import { getContract } from "thirdweb";
import { chainInfo } from "./chain_info";

// Replace this with your client ID string
// refer to https://portal.thirdweb.com/typescript/v5/client on how to get a client ID
const clientId = process.env.NEXT_PUBLIC_TEMPLATE_CLIENT_ID;

if (!clientId) {
  throw new Error("No client ID provided");
}

export const client = createThirdwebClient({
  clientId: clientId,
});

export const contract = getContract({
  client,
  chain: chainInfo,
  address: "0x0b301079281af307A3A02a334b3496339353EF27",
});