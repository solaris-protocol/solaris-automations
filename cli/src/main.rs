pub mod instructions;
use crate::instructions::*;

use std::{
    fs::File,
    error::Error,
    str::FromStr,
    io::Read,
};
use borsh::{
    BorshSerialize,
};
use solana_client::{ 
    rpc_client::RpcClient,
};
use solana_sdk::{
    instruction::{
        AccountMeta, 
        Instruction, 
    }, 
    signature::{
        Signer,
        read_keypair_file,
    },
    transaction::Transaction,
    pubkey::Pubkey,
};

pub const SETTINGS_PATH: &str = "settings.json";

fn main() -> Result<(), Box<dyn Error>>{
    let client = RpcClient::new("https://api.devnet.solana.com/".to_string());

    let settings = parse_settings_json(SETTINGS_PATH)?;
    let program_id = settings["program_id"].as_str().unwrap();
    let predicate_id = settings["predicate_id"].as_str().unwrap();
    let instruction = settings["instruction_num"].as_u64().unwrap();
    let payer_keypair = settings["payer_keypair"].as_str().unwrap();
    
    let payer_keypair = read_keypair_file(payer_keypair)
        .unwrap_or_else(|error| {
            panic!("Couldn't parse signer keypair: {}", error);
    });

    let program_id = Pubkey::from_str(program_id)?;
    let predicate_id = Pubkey::from_str(predicate_id)?;

    let instruction = match instruction {
        0 => {
            let accounts = vec![
                AccountMeta::new(payer_keypair.pubkey(), true), 
                AccountMeta::new(payer_keypair.pubkey(), false), 
                AccountMeta::new_readonly(predicate_id, false), 
            ];

            let predicate = bincode::serialize(
                &Instruction {
                    program_id: predicate_id,
                    accounts: vec![],
                    data: vec![0,1],
                })
                .unwrap();

            let data = SolarisAutoInstruction::FillOrder{
                predicate,
            }
            .try_to_vec().unwrap();

            Instruction {
                program_id,
                accounts,
                data,
            }
        }
        _ => panic!("Unexpected instruction")
    };

    let mut transaction = Transaction::new_with_payer(
        &[instruction],
        Some(&payer_keypair.pubkey()),
    );

    let blockhash = client.get_recent_blockhash()?.0;
    transaction.try_sign(&[&payer_keypair], blockhash)?;

    client.send_and_confirm_transaction_with_spinner(&transaction)?;

    Ok(())
}

pub fn parse_settings_json(
    path: &str
) -> Result<serde_json::Value, Box<dyn Error>> {
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