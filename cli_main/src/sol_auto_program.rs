use std::{
    error::Error,
    str::FromStr,
};
use clap::ArgMatches;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{
        Instruction, AccountMeta,
    }, 
    pubkey::Pubkey, 
    transaction::Transaction,
    keccak,
    signature::Signer,
};
use borsh::BorshSerialize;
use serde_json::Value;

use crate::instruction::*;
use super::{parse_json, parse_keypair};

pub const PREFIX: &str = "solaris-automations";
pub const ONCHAIN_ORDER: &str = "onchain_order";
pub const COLLATERAL_TA: &str = "collateral_ta_v2";
pub const DELEGATE: &str = "delegate";

pub fn send_fill_order(
    client: RpcClient,
    settings: Value,
    arg_matches: &ArgMatches,
) -> Result<(), Box<dyn Error>> {
    let program_id = Pubkey::from_str(settings["program_id"].as_str().unwrap())?; 
    let payer_keypair = parse_keypair(&settings, "payer_keypair")?;
    
    let payer_keypair_dalek = ed25519_dalek::Keypair::from_bytes(
        &payer_keypair.to_bytes())?;

    let taker_ta_maker_asset = settings["taker_ta_maker_asset"].as_str().unwrap();
    let maker_ta_maker_asset = settings["maker_ta_maker_asset"].as_str().unwrap();
    let taker_ta_taker_asset = settings["taker_ta_taker_asset"].as_str().unwrap();
    let maker_ta_taker_asset = settings["maker_ta_taker_asset"].as_str().unwrap();

    let taker_ta_maker_asset = Pubkey::from_str(taker_ta_maker_asset)?;
    let maker_ta_maker_asset = Pubkey::from_str(maker_ta_maker_asset)?;
    let taker_ta_taker_asset = Pubkey::from_str(taker_ta_taker_asset)?;
    let maker_ta_taker_asset = Pubkey::from_str(maker_ta_taker_asset)?;

    let delegate_id = get_pda_delegate_id(&program_id);

    let order_json = arg_matches.value_of("order").unwrap();
    let order_value = parse_json(order_json)?;

    let taking_amount = order_value["taking_amount"].as_u64().unwrap();
    let order = parse_order(&order_value);
    let (predicate_metas, callback_metas) = parse_metas(&order_value);
    let sign = parse_signature_instr(&order_value);

    println!("Maker is {:?}", order.maker.to_string());
    println!("Taker is {:?}", payer_keypair.pubkey().to_string());

    let order_hash = keccak::hash(&order.try_to_vec().unwrap());

    let onchain_order_id = get_pda_onchain_order(&program_id, order_hash.as_ref());

    let mut instructions = vec![];

    let (order_arg, order_stage) = match order.callback.is_empty() {
        true => {
            instructions.push(sign);
            (Some(order.clone()), OrderStage::Filled)
        },
        false => {
            match client.get_account(&onchain_order_id) {
                Ok(_) => (None, OrderStage::Filled),
                Err(_) => {
                    instructions.push(sign);
                    (Some(order.clone()), OrderStage::Create)
                },
            }
        },
    };

    println!("order.taking_amount is {}", order.taking_amount);

    instructions.push(
        fill_order(
            &program_id,
            &order.maker,
            &payer_keypair.pubkey(),
            &onchain_order_id,
            &delegate_id,
            predicate_metas, 
            callback_metas, 
            &taker_ta_taker_asset,
            &maker_ta_taker_asset,
            &taker_ta_maker_asset,
            &maker_ta_maker_asset,

            order_arg,
            0,
            taking_amount,
            0,

            order_stage,
        ),
    );

    let mut transaction = Transaction::new_with_payer(
        &instructions,
        Some(&payer_keypair.pubkey()),
    );

    let blockhash = client.get_recent_blockhash()?.0;
    transaction.try_sign(&[&payer_keypair], blockhash)?;

    client.send_and_confirm_transaction_with_spinner(&transaction)?;

    Ok(())
}

/* 
pub fn read_blockchain_config_by_id(
    client: RpcClient,
    settings: Value,
) -> Result<()> {
    let program_id = Pubkey::from_str(settings["program_id"].as_str().unwrap())?;
    let uuid = settings["uuid"].as_u64().unwrap();
    let blockchain_version = settings["blockchain_version"].as_u64().unwrap();

    let pda_blockchain_id = get_pda_blockchain_id(&program_id, uuid, blockchain_version);

    let data = client.get_account_data(&pda_blockchain_id)?;
    let blockchain_config = Blockchain::try_from_slice(&data)?;

    println!("Blockchain {:?} with uuid {} is:\n {:#?}", 
        pda_blockchain_id, uuid, blockchain_config);

    Ok(())
}
*/

pub fn get_pda_delegate_id(program_id: &Pubkey) -> Pubkey {
    let (delegate, bump) = Pubkey::find_program_address(
        &[PREFIX.as_bytes(), DELEGATE.as_bytes()],
        program_id,
    );

    println!("delegate is {:?} with bump is {}", delegate, bump);
    delegate
}

pub fn get_pda_onchain_order(program_id: &Pubkey, order_hash: &[u8]) -> Pubkey {
    let (onchain_order, _) = Pubkey::find_program_address(
        &[PREFIX.as_bytes(), ONCHAIN_ORDER.as_bytes(), order_hash],
        &program_id,
    );

    onchain_order
}

pub fn parse_order(
    order: &Value,
) -> Order {
    let salt = order["salt"].as_u64().unwrap();
    let maker_asset = order["maker_asset"].as_str().unwrap();
    let taker_asset = order["taker_asset"].as_str().unwrap();
    let maker = order["maker"].as_str().unwrap();
    let making_amount = order["making_amount"].as_u64().unwrap();
    let taking_amount = order["taking_amount"].as_u64().unwrap();
    //let get_maker_amount = order["get_maker_amount"].as_array().unwrap();
    //let get_taker_amount = order["get_taker_amount"].as_array().unwrap();
    let predicate = order["predicate"].as_array().unwrap();
    let callback = order["callback"].as_array().unwrap();

    let maker_asset = Pubkey::from_str(maker_asset).unwrap();
    let taker_asset = Pubkey::from_str(taker_asset).unwrap();
    let maker = Pubkey::from_str(maker).unwrap();
    /* 
    let get_maker_amount: Vec<u8> = get_maker_amount
        .iter()
        .map(|value| value.as_u64().unwrap() as u8)
        .collect();
    let get_taker_amount: Vec<u8> = get_taker_amount
        .iter()
        .map(|value| value.as_u64().unwrap() as u8)
        .collect();
    */
    let predicate: Vec<u8> = predicate
        .iter()
        .map(|value| value.as_u64().unwrap() as u8)
        .collect();
    let callback: Vec<u8> = callback
        .iter()
        .map(|value| value.as_u64().unwrap() as u8)
        .collect();

    Order {
        salt,
        maker_asset,
        taker_asset,
        maker,
        making_amount,
        taking_amount,
        predicate,
        callback,
    }
}

pub fn parse_metas(
    order: &Value,
) -> (Vec<AccountMeta>, Vec<AccountMeta>) {
    let predicate_metas = order["predicate_metas"].as_array().unwrap();
    let callback_metas = order["callback_metas"].as_array().unwrap();

    let parse_account_meta = |value: &Value| {
        let pubkey = value["pubkey"].as_str().unwrap();
        let is_signer = value["is_signer"].as_bool().unwrap();
        let is_writable = value["is_writable"].as_bool().unwrap();

        let pubkey = Pubkey::from_str(pubkey).unwrap();
        
        match is_writable {
            true => AccountMeta::new(pubkey, is_signer),
            false => AccountMeta::new_readonly(pubkey, is_signer),
        }
    };

    let predicate_metas: Vec<AccountMeta> = predicate_metas
        .iter()
        .map(parse_account_meta)
        .collect();

    let callback_metas: Vec<AccountMeta> = callback_metas
        .iter()
        .map(parse_account_meta)
        .collect();

    (predicate_metas, callback_metas)
}

pub fn parse_signature_instr(
    order: &Value,
) -> Instruction {
    let ed25519_instr = order["signature"].as_array().unwrap();

    let ed25519_instr: Vec<u8> = ed25519_instr
        .iter()
        .map(|value| value.as_u64().unwrap() as u8 )
        .collect();

    let sign = bincode::deserialize::<Instruction>(&ed25519_instr).unwrap();

    sign
}