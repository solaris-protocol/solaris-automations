pub mod instructions;
use crate::instructions::*;

use std::{
    fs::File,
    error::Error,
    str::FromStr,
    io::Read,
};
use borsh::{
    BorshSerialize, BorshDeserialize, BorshSchema,
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
pub const ONCHAIN_ORDER: &str = "onchain_order";
pub const DELEGATE: &str = "delegate";

#[derive(BorshDeserialize, BorshSchema, BorshSerialize)]
pub struct HelperPythPrice {
    amount: u64,
    less_than_pyth_price: bool,
}

fn main() -> Result<(), Box<dyn Error>>{
    let client = RpcClient::new("https://api.mainnet-beta.solana.com/".to_string());

    let settings = parse_settings_json(SETTINGS_PATH)?;
    let program_id = settings["program_id"].as_str().unwrap();
    let predicate_id = settings["predicate_id"].as_str().unwrap();
    let custom_get_amounts = settings["custom_get_amounts"].as_str().unwrap();
    let pyth_price = settings["pyth_price_id"].as_str().unwrap();
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

    let making_amount = settings["making_amount"].as_u64().unwrap();
    let taking_amount = settings["taking_amount"].as_u64().unwrap();

    let order_stage = settings["order_stage"].as_str().unwrap();

    let salt = settings["salt"].as_u64().unwrap();

    let solend_program_id = settings["solend_program_id"].as_str().unwrap();

    let solend_reserve = settings["solend_reserve"].as_str().unwrap();
    let solend_pyth_price = settings["solend_pyth_price"].as_str().unwrap();
    let solend_switchboard_price = settings["solend_switchboard_price"].as_str().unwrap();

    let solend_obligation = settings["solend_obligation"].as_str().unwrap();

    let solend_source_withdraw_reserve_collateral = settings["solend_source_withdraw_reserve_collateral"].as_str().unwrap();
    let solend_destination_collateral = settings["solend_destination_collateral"].as_str().unwrap();
    let solend_lending_market = settings["solend_lending_market"].as_str().unwrap();
    let solend_derived_lending_market_authority = settings["solend_derived_lending_market_authority"].as_str().unwrap();
    let solend_user_liquidity_token_account = settings["solend_user_liquidity_token_account"].as_str().unwrap();
    let solend_reserve_collateral_mint = settings["solend_reserve_collateral_mint"].as_str().unwrap();
    let solend_reserve_liquidity_token_account = settings["solend_reserve_liquidity_token_account"].as_str().unwrap();

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

    let obligation_owner = settings["obligation_owner"].as_str().unwrap();
    let obligation_owner = read_keypair_file(obligation_owner)
        .unwrap_or_else(|error| {
            panic!("Couldn't parse maker keypair: {}", error);
    });

    let program_id = Pubkey::from_str(program_id)?;
    let predicate_id = Pubkey::from_str(predicate_id)?;
    let custom_get_amounts = Pubkey::from_str(custom_get_amounts)?;
    let pyth_price = Pubkey::from_str(pyth_price)?;
    let helper_and_id = Pubkey::from_str(helper_and_id)?;
    let maker_asset = Pubkey::from_str(maker_asset)?;
    let taker_asset = Pubkey::from_str(taker_asset)?;

    let taker_ta_maker_asset = Pubkey::from_str(taker_ta_maker_asset)?;
    let maker_ta_maker_asset = Pubkey::from_str(maker_ta_maker_asset)?;
    let taker_ta_taker_asset = Pubkey::from_str(taker_ta_taker_asset)?;
    let maker_ta_taker_asset = Pubkey::from_str(maker_ta_taker_asset)?;

    let solend_program_id = Pubkey::from_str(solend_program_id)?;

    let solend_reserve = Pubkey::from_str(solend_reserve)?;
    let solend_pyth_price = Pubkey::from_str(solend_pyth_price)?;
    let solend_switchboard_price = Pubkey::from_str(solend_switchboard_price)?;

    let solend_obligation = Pubkey::from_str(solend_obligation)?;

    let solend_source_withdraw_reserve_collateral = Pubkey::from_str(solend_source_withdraw_reserve_collateral)?;
    let solend_destination_collateral = Pubkey::from_str(solend_destination_collateral)?;
    let solend_lending_market = Pubkey::from_str(solend_lending_market)?;
    let solend_derived_lending_market_authority = Pubkey::from_str(solend_derived_lending_market_authority)?;
    let solend_user_liquidity_token_account = Pubkey::from_str(solend_user_liquidity_token_account)?;
    let solend_reserve_collateral_mint = Pubkey::from_str(solend_reserve_collateral_mint)?;
    let solend_reserve_liquidity_token_account = Pubkey::from_str(solend_reserve_liquidity_token_account)?;

    let order_stage = match order_stage {
        "Create" => OrderStage::Create,
        "Filled" => OrderStage::Filled,
        _ => panic!("Unexpected order_stage")
    };

    let instruction = match instruction {
        0 => {
            let helper_pyth_istr_data = HelperPythPrice {
                amount: 100_000_000_000,
                less_than_pyth_price: false,
            }
            .try_to_vec().unwrap();

            let instr_and = bincode::serialize(&vec![
                Instruction{
                    program_id: predicate_id,
                    accounts: vec![],
                    data: helper_pyth_istr_data.clone(),
                },
                Instruction{
                    program_id: predicate_id,
                    accounts: vec![],
                    data: helper_pyth_istr_data.clone(),
                },
            ]).unwrap();

            let predicate = bincode::serialize(
                &Instruction{   
                    program_id: predicate_id,
                    accounts: vec![],
                    data: vec![0],
                })
                .unwrap();

            let get_maker_amount = bincode::serialize(
                &Instruction{
                    program_id: custom_get_amounts,
                    accounts: vec![],
                    data: vec![0], // doesn't deserialized with empty data
                })
                .unwrap();

            let order = Order {
                salt: salt,
                maker_asset,
                taker_asset,
                maker: maker_keypair.pubkey(),
                receiver: maker_keypair.pubkey(),
                allowed_sender: maker_keypair.pubkey(),
                making_amount: 2_000_000_000,
                taking_amount: 2_000_000_000,
                get_maker_amount,
                get_taker_amount: vec![],
                predicate,
                callback: vec![0],
            };

            let order_hash = keccak::hash(&order.try_to_vec().unwrap());
            let signature_inst = ed25519_instruction::new_ed25519_instruction(
                &maker_keypair_dalek, order_hash.as_ref(),
            );

            vec![
                //signature_inst, 
                fill_order(
                    &program_id,
                    &maker_keypair.pubkey(),
                    &taker_keypair.pubkey(),
                    &get_pda_onchain_order(&program_id, order_hash.as_ref()),
                    &get_pda_delegate_id(&program_id),
                    //&[custom_get_amounts],
                    &[],
                    //&[custom_get_amounts],
                    &[],
                    &[predicate_id, pyth_price],
                    &[predicate_id], //callback
                    &taker_ta_taker_asset, 
                    &maker_ta_taker_asset, 
                    &taker_ta_maker_asset,
                    &maker_ta_maker_asset, 

                    None,
                    making_amount,
                    taking_amount,
                    0,

                    order_stage,
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
        2 => {
            let refresh_reserve = Instruction{
                program_id: solend_program_id,
                accounts: vec![
                    AccountMeta::new(solend_reserve, false),
                    AccountMeta::new_readonly(solend_pyth_price, false),
                    AccountMeta::new_readonly(solend_switchboard_price, false),
                    AccountMeta::new_readonly(sysvar::clock::ID, false),
                ],
                data: vec![3],
            };

            let refresh_obligation = Instruction{
                program_id: solend_program_id,
                accounts: vec![
                    AccountMeta::new(solend_obligation, false),
                    AccountMeta::new_readonly(sysvar::clock::ID, false),
                    AccountMeta::new_readonly(solend_reserve, false),
                ],
                data: vec![7],
            };

            let withdraw_obligation_collaterial_and_redeem_reserve_collaterial = 
                Instruction{
                    program_id: solend_program_id,
                    accounts: vec![
                        AccountMeta::new(solend_source_withdraw_reserve_collateral, false),
                        AccountMeta::new(solend_destination_collateral, false),
                        AccountMeta::new(solend_reserve, false),
                        AccountMeta::new(solend_obligation, false),
                        AccountMeta::new_readonly(solend_lending_market, false),
                        AccountMeta::new_readonly(solend_derived_lending_market_authority, false),
                        AccountMeta::new(solend_user_liquidity_token_account, false),
                        AccountMeta::new(solend_reserve_collateral_mint, false),
                        AccountMeta::new(solend_reserve_liquidity_token_account, false),
                        AccountMeta::new(obligation_owner.pubkey(), true),
                        AccountMeta::new(obligation_owner.pubkey(), true),
                        AccountMeta::new_readonly(sysvar::clock::ID, false),
                        AccountMeta::new_readonly(spl_token::ID, false),
                    ],
                    data: vec![15, 255, 255, 255, 255, 255, 255, 255, 255],
                 };

            vec![
                refresh_reserve,
                refresh_obligation,
                withdraw_obligation_collaterial_and_redeem_reserve_collaterial,
            ]
        }
        _ => panic!("Unexpected instruction")
    };

    let mut transaction = Transaction::new_with_payer(
        &instruction,
        Some(&obligation_owner.pubkey()),
    );

    let blockhash = client.get_recent_blockhash()?.0;
    transaction.try_sign(&[&obligation_owner], blockhash)?;

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

pub fn get_pda_onchain_order(program_id: &Pubkey, order_hash: &[u8]) -> Pubkey {
    let (onchain_order, _) = Pubkey::find_program_address(
        &[PREFIX.as_bytes(), ONCHAIN_ORDER.as_bytes(), order_hash],
        &program_id,
    );

    onchain_order
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