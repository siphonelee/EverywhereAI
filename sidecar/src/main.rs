use ethers::prelude::*;
use ethers::providers::{Provider, Http};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::atomic::Ordering; 
use regex::Regex;
use timeout_readwrite::TimeoutReader;
use std::os::unix::io::AsRawFd;
use std::time::Duration;
use std::{
    io::{stderr, stdout, Read, Write},
    process::{Command, Stdio},
    thread,
    thread::JoinHandle,
    result::Result,
    error::Error,
};
use signal_hook::flag;
use signal_hook::consts::TERM_SIGNALS;
use std::marker::{Send, Sync};
use std::fmt::Debug;
use tokio::time::{sleep};

#[macro_use]
extern crate dotenv_codegen;

use ethers_core::{
     types::{Address},
};

abigen!(IAgent, "./src/Agent.json");

fn parse_log(stream: impl Read+AsRawFd, mut _output: impl Write, stop_flag: Arc<AtomicBool>, accumulate_tokens: Arc<AtomicU64>) -> Result<(), Box<dyn Error>> {
    let flag = "server listening at";
    let re_url = Regex::new(r"(http:\/\/.*)").unwrap();
    let re_tokens = Regex::new(r###""n_decoded":(\d+),"###).unwrap();

    let mut s_output = String::new();
    let mut rdr = TimeoutReader::new(stream, Duration::new(2, 0));
    let mut buf = [0u8; 1024];
    loop {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        if let Ok(num_read) = rdr.read(&mut buf) {
            if num_read == 0 {
                break;
            }

            let buf = &buf[..num_read];
            // _output.write_all(buf)?;
            let s = std::str::from_utf8(buf).unwrap();
            s_output.push_str(s);
            if s_output.contains(flag) {
                for (_, [url]) in re_url.captures_iter(s_output.as_str()).map(|c| c.extract()) {
                    println!("LLM service started at {}", url);
                    // stop_flag.store(true, Ordering::Relaxed);
                }

                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
            } else if s_output.contains("{\"function\":\"print_timings\",") {
                for line in s_output.lines() {
                    for (_, [tokens]) in re_tokens.captures_iter(line).map(|c| c.extract()) {
                        println!("tokens: {}", tokens);
                        accumulate_tokens.fetch_add(tokens.parse::<u64>()?, Ordering::SeqCst);
                    }
                } 
            }

            if let Some((_, t)) = s_output.clone().rsplit_once("\n") {
                s_output.clear();
                s_output.push_str(t);
            }

        } else {
            continue;
        }
    }
    
    Ok(())
}

fn start_llm(accumulate_tokens: Arc<AtomicU64>, stop_flag: Arc<AtomicBool>) -> Result<(std::process::Child, JoinHandle<()>, JoinHandle<()>), Box<dyn Error>> {
    for sig in TERM_SIGNALS {
        flag::register(*sig, stop_flag.clone())?;
    }

    let mut child = Command::new("./Meta-Llama-3-8B-Instruct.Q5_K_M.llamafile")
        .args(["--nobrowser", "--unsecure"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start LLM as child");

    let child_out = std::mem::take(&mut child.stdout).expect("cannot attach to child stdout");
    let child_err = std::mem::take(&mut child.stderr).expect("cannot attach to child stderr");

    let stop_flag1 = stop_flag.clone();
    let accumulate_tokens1 = accumulate_tokens.clone();
    let thread_out = thread::spawn(move || {
        parse_log(child_out, stdout(), stop_flag1, accumulate_tokens1)
            .expect("error communicating with child stdout")
    }); 

    let stop_flag2 = stop_flag.clone();
    let accumulate_tokens2 = accumulate_tokens.clone();
    let thread_err = thread::spawn(move || {
        parse_log(child_err, stderr(), stop_flag2, accumulate_tokens2)
            .expect("error communicating with child stderr")
    });

    Ok((child, thread_out, thread_err))
}

fn start_record_tokens<T>(contract: Arc<IAgent<T>>, accumulate_tokens: Arc<AtomicU64>, stop_flag: Arc<AtomicBool>) -> Result<tokio::task::JoinHandle<()>, Box<dyn Error>> 
                          where T: ethers_middleware::Middleware+Debug+Sync+Send+'static {
    let thread = tokio::spawn(async move {
        loop {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            let tokens = accumulate_tokens.swap(0u64, Ordering::SeqCst);
            if tokens > 0 {
                let tx = contract.increase_credit(U256::from(tokens)).send().await.unwrap().await.unwrap();
                let event_log = &tx.unwrap().logs[0];
                let inc_tokens = U256::from_big_endian(&event_log.data[0..32]);
                let total_tokens = U256::from_big_endian(&event_log.data[32..64]);
                println!("added tokens: {}, total tokens: {}", inc_tokens, total_tokens);
            } else {
                sleep(tokio::time::Duration::from_secs(1)).await;        
            }
        }
    });

    Ok(thread)    
}

fn parse_contract_result(s: String) -> Result<String, String> {
   if s.contains("transactionHash") {
       let re_hash = Regex::new(r###""transactionHash":"(.*?)","###).unwrap();
       for (_, [hash]) in re_hash.captures_iter(s.as_str()).map(|c| c.extract()) {
           return Ok(hash.to_owned());
       }
       Err(s)
   } else {
       Err(s) 
   }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let contract_address = "0xFd35805FECF1d928ec753bfc0e2AFa1068124fe4".parse::<Address>()?;

    let rpc_url = format!("https://linea-sepolia.blockpi.network/v1/rpc/public");
    let provider = Provider::<Http>::try_from(rpc_url.as_str())?;
    let chain_id = provider.get_chainid().await?;

    let wallet = dotenv!("WALLET_PRIV_KEY").parse::<LocalWallet>()?;
    let wallet = wallet.with_chain_id(chain_id.as_u64());
    let client = SignerMiddleware::new(provider, wallet);

    let provider = Arc::new(client);
    let contract = Arc::new(IAgent::new(contract_address, provider.clone()));

    let stop_flag = Arc::new(AtomicBool::new(false));

    println!("Registering to contract...");
    let tx = contract.register(dotenv!("REGISTER_URL").to_owned()).send().await?.await?;
    match parse_contract_result(serde_json::to_string(&tx)?) {
        Ok(hash) => println!("Successfully registered, transaction hash: {}", hash),
        Err(err) => println!("Registering failed: {}", err),
    }
 
    let accumulate_tokens = Arc::new(AtomicU64::new(0));

    println!("Starting LLM service...");
    let (mut llm_process, t1, t2) = start_llm(accumulate_tokens.clone(), stop_flag.clone())?;

    let t3 = start_record_tokens(contract.clone(), accumulate_tokens, stop_flag)?;

    t1.join().unwrap();
    t2.join().unwrap();
    let _ = llm_process.kill();
    
    t3.await?;

    println!("Unregistering from contract...");
    let tx = contract.unregister().send().await?.await?;
    match parse_contract_result(serde_json::to_string(&tx)?) {
        Ok(hash) => println!("Successfully registered, transaction hash: {}", hash),
        Err(err) => println!("Registering failed: {}", err),
    }

    Ok(())
}
