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
    sysvar,
    ed25519_instruction, 
    keccak::{Hasher, Hash, hash, self},
};
use ed25519_dalek::Keypair;

pub const SETTINGS_PATH: &str = "settings.json";

pub const PREFIX: &str = "solaris-automations";
pub const DELEGATE: &str = "delegate";

fn main() -> Result<(), Box<dyn Error>>{
    let client = RpcClient::new("https://api.devnet.solana.com/".to_string());

    let settings = parse_settings_json(SETTINGS_PATH)?;
    let program_id = settings["program_id"].as_str().unwrap();
    let predicate_id = settings["predicate_id"].as_str().unwrap();
    let helper_and_id = settings["helper_and_id"].as_str().unwrap();
    let instruction = settings["instruction_num"].as_u64().unwrap();
    let maker_keypair = settings["maker_keypair"].as_str().unwrap();
    let taker_keypair = settings["taker_keypair"].as_str().unwrap();
    let maker_asset = settings["maker_asset"].as_str().unwrap();
    let taker_asset = settings["taker_asset"].as_str().unwrap();

    let taker_ta_maker_asset = settings["taker_ta_maker_asset"].as_str().unwrap();
    let maker_ta_maker_asset = settings["maker_ta_maker_asset"].as_str().unwrap();
    let taker_ta_taker_asset = settings["taker_ta_taker_asset"].as_str().unwrap();
    let maker_ta_taker_asset = settings["maker_ta_taker_asset"].as_str().unwrap();
    
    let maker_keypair = read_keypair_file(maker_keypair)
        .unwrap_or_else(|error| {
            panic!("Couldn't parse maker keypair: {}", error);
    });

    let taker_keypair = read_keypair_file(taker_keypair)
        .unwrap_or_else(|error| {
            panic!("Couldn't parse taker keypair: {}", error);
    });

    let maker_keypair_dalek = ed25519_dalek::Keypair::from_bytes(
        &maker_keypair.to_bytes())?;

    let program_id = Pubkey::from_str(program_id)?;
    let predicate_id = Pubkey::from_str(predicate_id)?;
    let helper_and_id = Pubkey::from_str(helper_and_id)?;
    let maker_asset = Pubkey::from_str(maker_asset)?;
    let taker_asset = Pubkey::from_str(taker_asset)?;

    let taker_ta_maker_asset = Pubkey::from_str(taker_ta_maker_asset)?;
    let maker_ta_maker_asset = Pubkey::from_str(maker_ta_maker_asset)?;
    let taker_ta_taker_asset = Pubkey::from_str(taker_ta_taker_asset)?;
    let maker_ta_taker_asset = Pubkey::from_str(maker_ta_taker_asset)?;

    let instruction = match instruction {
        0 => {
            let instr_and = bincode::serialize(&vec![
                Instruction{
                    program_id: predicate_id,
                    accounts: vec![],
                    data: vec![0,1],
                },
                Instruction{
                    program_id: predicate_id,
                    accounts: vec![],
                    data: vec![0,1],
                },
            ]).unwrap();

            let predicate = bincode::serialize(
                &Instruction{
                    program_id: helper_and_id,
                    accounts: vec![],
                    data: instr_and,
                })
                .unwrap();

            let order = Order {
                salt: 0,
                maker_asset,
                taker_asset,
                maker: maker_keypair.pubkey(),
                receiver: maker_keypair.pubkey(),
                allowed_sender: maker_keypair.pubkey(),
                making_amount: 1_000_000_000,
                taking_amount: 2_000_000_000,
                get_maker_amount: vec![],
                get_taker_amount: vec![],
                predicate,
                interaction: vec![],
            };

            let signature_inst = ed25519_instruction::new_ed25519_instruction(
                &maker_keypair_dalek, keccak::hash(&order.try_to_vec().unwrap()).as_ref(),
            );

            vec![
                signature_inst, 
                fill_order(
                    &program_id,
                    &maker_keypair.pubkey(),
                    &taker_keypair.pubkey(),
                    &get_pda_delegate_id(&program_id),
                    &[predicate_id],
                    &[maker_ta_maker_asset, taker_ta_maker_asset],
                    &[taker_ta_taker_asset, maker_ta_taker_asset, taker_keypair.pubkey()],

                    order,
                    1_000_000_000,
                    2_000_000_000,
                    0,
                )]
        },
        1 => {
            let init_delegate = init_delegate(
                &program_id,
                &taker_keypair.pubkey(),
                &get_pda_delegate_id(&program_id),
            );

            vec![init_delegate]
        },
        _ => panic!("Unexpected instruction")
    };

    let mut transaction = Transaction::new_with_payer(
        &instruction,
        Some(&taker_keypair.pubkey()),
    );

    let blockhash = client.get_recent_blockhash()?.0;
    transaction.try_sign(&[&taker_keypair], blockhash)?;

    client.send_and_confirm_transaction_with_spinner(&transaction)?;

    Ok(())
}

pub fn get_pda_delegate_id(program_id: &Pubkey) -> Pubkey {
    let (delegate, bump) = Pubkey::find_program_address(
        &[PREFIX.as_bytes(), DELEGATE.as_bytes()],
        program_id,
    );

    println!("bump delegate is {}", bump);
    delegate
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