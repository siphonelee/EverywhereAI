import { createThirdwebClient } from "thirdweb";
import { getContract } from "thirdweb";
import { lineaSepolia } from "./linea-sepolia";

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
  chain: lineaSepolia,
  address: "0xFd35805FECF1d928ec753bfc0e2AFa1068124fe4",
});
