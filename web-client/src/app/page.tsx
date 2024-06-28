'use client'

import Image from "next/image";
import { ConnectButton, useReadContract } from "thirdweb/react";
import thirdwebIcon from "@public/thirdweb.svg";
import { client, contract } from "./client";

export default function Home() {
  return (
    <main className="p-4 pb-10 items-center justify-center container  mx-auto">
      <div className="py-20">
        <Header />
        <AIResources />
      </div>
    </main>
  );
}

function Header() {
  return (
    <header className="flex flex-row-reversed items-center mb-20 md:mb-20">
      <h1 className="text-2xl md:text-6xl font-semibold md:font-bold tracking-tighter mb-6 text-zinc-100">
        Everywhere AI
      </h1>
      <div className="flex-grow"/>
      <div className="flex justify-center mb-6">
          <ConnectButton
            client={client}
            appMetadata={{
              name: "AI Agent App",
              url: "http://localhost:3000",
            }}
          />
        </div>
    </header>
  );
}

function AIResources() {
  const { data: url, isLoading } = useReadContract({
    contract, 
    method: "function getUrl() external view returns (string memory)", 
    params: []
  });

  return (
    <div className="grid gap-4 lg:grid-cols-3 justify-center aspect-w-16 aspect-h-12">
      <iframe className="w-full h-full min-h-full" width="100%" height="100%" src={isLoading || url == null? "" : url}>
      </iframe>    
    </div>
  );  
}

