pub mod parse_args;
use crate::parse_args::get_arg_matches;

pub mod sol_auto_program;
use sol_auto_program::*;

pub mod instruction;
pub mod create_order;
use create_order::*;

use std::{ 
    net::{ SocketAddr, },
    error::{ Error, },
    time::{ Duration, },
    fs::File,
    io::prelude::*,
};
use anyhow::{
    Context,
    Result,
    anyhow,
};
use solana_client::{ 
    rpc_client::RpcClient,
};
use solana_sdk::signer::keypair::{Keypair, read_keypair_file};
use serde_json::Value;

fn main() -> Result<(), Box<dyn Error>> {
    let app_matches = get_arg_matches();

    let clap_config = app_matches.value_of("config").unwrap_or("default.conf");
    println!("Value for config: {}", clap_config);

    let settings_path = app_matches.value_of("settings").unwrap();
    let settings = parse_json(settings_path).unwrap_or_else(|error| {
        panic!("Couldn't parse json with settings: {}", error);
    });

    let client = match settings["cluster"].as_str().unwrap() {
        "devnet" =>  RpcClient::new("https://api.devnet.solana.com/".to_string()),
        "testnet" =>  RpcClient::new("https://api.testnet.solana.com/".to_string()),
        "mainnet" =>  RpcClient::new("https://api.mainnet-beta.solana.com/".to_string()),
        "localhost" => {
            let rpc_addr: &str = "127.0.0.1:8899";  
            let timeout = 1000;
            let rpc_addr: SocketAddr = rpc_addr.parse().expect("");

            RpcClient::new_socket_with_timeout(
                rpc_addr, Duration::from_millis(timeout))
        },
        _ => unreachable!()
    };

    let (sub_command, args) = app_matches.subcommand();

    let _ = match sub_command {
        "fill_order" => {
            println!("Solaris-automation program: FillOrder");
            send_fill_order(client, settings, args.unwrap())?;
        },
        "create_order" => {
            println!("Solaris-automation cli: CreateOrder");
            create_order(args.unwrap());
        },
        _ => unreachable!(),
    };

    Ok(())
}

pub fn parse_json(
    path: &str
) -> Result<serde_json::Value> {
    let mut file = File::open(path).unwrap_or_else(|error| {
        panic!("Couldn't open file: {}", error);
    });

    let mut data = String::new();
    let _ = file.read_to_string(&mut data).unwrap_or_else(|error| {
        panic!("Couldn't read data: {}", error);
    });

    let v: serde_json::Value = serde_json::from_str(&data).unwrap();

    Ok(v)
}

pub fn parse_keypair(
    settings: &Value,
    name: &str,
) -> Result<Keypair, Box<dyn Error>> {
    let path = settings[name].as_str().unwrap();
    let keypair = read_keypair_file(path)
        .map_err(|e| anyhow!("{}", e))
        .context("unable to load keypair")?;

    Ok(keypair)
}