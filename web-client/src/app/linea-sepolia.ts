import { Chain } from "thirdweb/chains";

export const lineaSepolia: Chain = {
    "id": 59141,
    "blockExplorers": [
      {
        "name": "Etherscan",
        "url": "https://sepolia.lineascan.build",
      }
    ],
    "name": "Linea Sepolia",
    "nativeCurrency": {
      "name": "Linea Ether",
      "symbol": "ETH",
      "decimals": 18
    },
    "rpc": "https://rpc.sepolia.linea.build",
    "testnet": true,
  }