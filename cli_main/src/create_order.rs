use std::{
    error::Error,
    str::FromStr,
    fs::File,
};
use solana_sdk::{
    pubkey::Pubkey, 
    instruction::{AccountMeta, Instruction},
    clock::Clock,
    sysvar::SysvarId, 
    ed25519_instruction, 
    signature::Signer,
    keccak, 
};
use borsh::BorshSerialize;
use serde_json::Value;
use serde::{Serialize, Deserialize};
use byteorder::ByteOrder;
use clap::ArgMatches;
use rand::{RngCore, rngs::OsRng};
use thiserror::Error;
use crate::instruction::Order;

use super::{parse_json, parse_keypair};


pub const HELPER_PYTH_ID: &[u8] = &[70, 176, 37, 12, 106, 201, 74, 156, 64, 246, 254, 0, 195, 85, 90, 97, 88, 148, 195, 146, 24, 6, 246, 114, 86, 228, 185, 63, 193, 54, 105, 176];
pub const PREDICATE_HEALTHFACTOR_ID: &[u8] = &[78, 11, 118, 213, 228, 92, 26, 55, 101, 204, 11, 75, 138, 91, 78, 249, 10, 197, 229, 133, 84, 247, 212, 213, 21, 232, 235, 119, 192, 110, 179, 177];

pub const CALLBACK_SOLEND_LIQUIDATION_PROTECTION: &[u8] = &[32, 53, 9, 1, 169, 162, 247, 15, 108, 3, 155, 52, 149, 20, 2, 86, 154, 148, 207, 4, 134, 152, 207, 14, 16, 80, 168, 73, 173, 86, 193, 182];

pub const SOLEND_ID: &[u8] = &[138, 193, 241, 114, 69, 245, 144, 57, 23, 131, 163, 184, 86, 117, 180, 107, 157, 175, 93, 163, 95, 242, 88, 210, 223, 21, 247, 109, 180, 231, 50, 89];


#[derive(Serialize, Deserialize, Debug)]
pub struct OrderBase {
    salt: u64,
    maker_asset: String,
    taker_asset: String,
    maker: String,
    making_amount: u64,
    taking_amount: u64,
    predicate: Vec<u8>,
    callback: Vec<u8>,
    predicate_metas: Vec<CustomAccountMeta>,
    callback_metas: Vec<CustomAccountMeta>,
    signature: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomAccountMeta {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Error, Debug)]
pub enum CreateOrderError {
    #[error("Empty")]
    Empty,
}

pub fn create_order(arg_matches: &ArgMatches) {
    let order_base = arg_matches.value_of("order_base").unwrap();
    let order_base = parse_json(order_base).unwrap();

    let maker = parse_keypair(&order_base, "maker").unwrap();
    let maker_dalek = ed25519_dalek::Keypair::from_bytes(
        &maker.to_bytes()).unwrap();

    let maker_asset = order_base["maker_asset"].as_str().unwrap();
    let taker_asset = order_base["taker_asset"].as_str().unwrap();
    let making_amount = order_base["making_amount"].as_u64().unwrap();
    let taking_amount = order_base["taking_amount"].as_u64().unwrap();

    let predicate = parse_predicate(&order_base);
    let (predicate, predicate_metas) = match predicate {
        Ok(predicate) => {
            let mut predicate_metas = predicate.accounts.clone();
            predicate_metas.insert(0, AccountMeta::new(predicate.program_id, false));
        
            let predicate_custom_metas = fill_custom_metas(predicate_metas);
            let predicate = bincode::serialize(&predicate).unwrap();

            (predicate, predicate_custom_metas)
        },
        Err(_) => (vec![], vec![])
    };

    let callback = parse_callback(&order_base);
    let (callback, callback_metas) = match callback {
        Ok(callback) => {
            let mut callback_metas = callback.accounts.clone();
            callback_metas.insert(0, AccountMeta::new(callback.program_id, false));

            let callback_custom_metas = fill_custom_metas(callback_metas);
            let callback = bincode::serialize(&callback).unwrap();

            (callback, callback_custom_metas)
        },
        Err(_) => (vec![], vec![])
    };

    let mut order_base = OrderBase {
        salt: OsRng.next_u64(),
        maker_asset: maker_asset.to_string(),
        taker_asset: taker_asset.to_string(),
        maker: maker.pubkey().to_string(),
        making_amount,
        taking_amount,
        predicate,
        callback,
        predicate_metas,
        callback_metas,
        signature: vec![],
    };

    let order = order_base.to_order();

    let order_hash = keccak::hash(&order.try_to_vec().unwrap());

    let signature_instr = ed25519_instruction::new_ed25519_instruction(
        &maker_dalek, order_hash.as_ref(),
    );  

    order_base.signature = bincode::serialize(&signature_instr).unwrap();

    serde_json::to_writer_pretty(&File::create("order_test.json").unwrap(), &order_base);
}

fn parse_predicate(order_base: &Value) -> Result<Instruction, Box<dyn Error>> {
    let predicate = order_base["predicate"].as_str().ok_or(CreateOrderError::Empty)?;

    let mut instruction = Instruction {
        program_id: Pubkey::new_unique(),
        accounts: vec![],
        data: vec![],
    };

    let mut custom_account_metas: Vec<CustomAccountMeta> = vec![];

    match predicate {
        "oracle_price" => {
            let predicate_oracle = order_base["predicate_oracle"].as_str().unwrap();

            match predicate_oracle {
                "pyth" => {
                    instruction.program_id = Pubkey::new(HELPER_PYTH_ID);

                    let predicate_condition = order_base["predicate_condition"].as_str().unwrap();
                    let predicate_pyth_price = order_base["predicate_pyth_price"].as_str().unwrap();

                    let pyth_price_id = Pubkey::from_str(predicate_pyth_price).unwrap();

                    instruction.accounts = vec![
                        AccountMeta::new_readonly(pyth_price_id, false),
                    ];

                    match predicate_condition {
                        "less" => {
                            let amount = order_base["predicate_price"].as_u64().unwrap();

                            let mut data: [u8; 9] = [0; 9];
                            byteorder::LE::write_u64(&mut data[0..8], amount);
                            data[8] = 1;

                            instruction.data = data.to_vec();
                        },
                        "more" => {
                            let amount = order_base["predicate_price"].as_u64().unwrap();

                            let mut data: [u8; 9] = [0; 9];
                            byteorder::LE::write_u64(&mut data[0..8], amount);
                            data[8] = 0;

                            instruction.data = data.to_vec();
                        },
                        _ => panic!("Unexpected predicate_condition")
                    }
                },
                _ => panic!("Unexpected predicate_oracle name")
            }
        },
        "lending_healthfactor" => {
            let lending_protocol = order_base["predicate_lending_protocol"].as_str().unwrap();

            match lending_protocol {
                "solend" => {
                    instruction.program_id = Pubkey::new(PREDICATE_HEALTHFACTOR_ID);
                    
                    let obligation_account_id = Pubkey::from_str("6FewVDiMS42WP31jQJP3S2WrEmyG5VHCjemt9YyjDJTi").unwrap();
                    
                    instruction.accounts = vec![
                        AccountMeta::new_readonly(obligation_account_id, false),
                    ];

                    let healthfactor = order_base["predicate_healthfactor"].as_u64().unwrap();

                    // healthfactor is interest that represents in 1e-6
                    let mut data: [u8; 16] = [0; 16];
                    byteorder::LE::write_u128(
                        &mut data[0..16], 
                        (healthfactor as u128 * 1_000_000_000_000_000_000 as u128) as u128,
                    );

                    instruction.data = data.to_vec();
                },
                _ => panic!("Unexpected lending protocol")
            }
        }
        _ => panic!("Unexpected predicate name")
    }

    Ok(instruction)
}


fn parse_callback(order_base: &Value) -> Result<Instruction, Box<dyn Error>> {
    let callback = order_base["callback"].as_str().ok_or(CreateOrderError::Empty)?;

    let mut instruction = Instruction {
        program_id: Pubkey::new_unique(),
        accounts: vec![],
        data: vec![],
    };

    let mut custom_account_metas: Vec<CustomAccountMeta> = vec![];

    match callback {
        "liquidation_protection" => {
            let callback_lending_protocol = order_base["callback_lending_protocol"].as_str().unwrap();

            match callback_lending_protocol {
                "solend" => {
                    instruction.program_id = Pubkey::new(CALLBACK_SOLEND_LIQUIDATION_PROTECTION);

                    // Hardcode for USDC 
                    let solend_program = Pubkey::new(SOLEND_ID);
                    // maker t-a for devnet USDC solend
                    let source_liquidity_token_account = Pubkey::from_str("AA3xpXu6j5JbnLVjekHhE39d4fXuJvusJbroinFciiEP").unwrap();
                    let dest_reserve_liquidity = Pubkey::from_str("HixjFJoeD2ggqKgFHQxrcJFjVvE5nXKuUPYNijFg7Kc5").unwrap();
                    let dest_reserve_account = Pubkey::from_str("FNNkz4RCQezSSS71rW2tvqZH1LCkTzaiG7Nd1LeA5x5y").unwrap();
                    // obligation for maker my-keypair.json
                    let obligation_account = Pubkey::from_str("6FewVDiMS42WP31jQJP3S2WrEmyG5VHCjemt9YyjDJTi").unwrap();
                    let lending_market = Pubkey::from_str("GvjoVKNjBvQcFaSKUW1gTE7DxhSpjHbE69umVR5nPuQp").unwrap();
                    // delegate
                    let transfer_authority = Pubkey::from_str("B4jsXjaAdFPEti6nRGFTwAb2NFaQ7tLriDoryBHUjzfy").unwrap();
                    
                    instruction.accounts = vec![
                        AccountMeta::new_readonly(solend_program, false),
                        AccountMeta::new(source_liquidity_token_account, false),
                        AccountMeta::new(dest_reserve_liquidity, false),
                        AccountMeta::new(dest_reserve_account, false),
                        AccountMeta::new(obligation_account, false),
                        AccountMeta::new_readonly(lending_market, false),
                        AccountMeta::new(transfer_authority, false),
                        AccountMeta::new_readonly(Clock::id(), false),
                        AccountMeta::new_readonly(spl_token::id(), false),
                    ];

                    let amount = order_base["taking_amount"].as_u64().unwrap();

                    let mut data: [u8; 9] = [0; 9];
                    data[0] = 11;
                    byteorder::LE::write_u64(&mut data[1..9], amount);

                    instruction.data = data.to_vec();
                },
                _ => panic!("Unexpected lending protocol")
            }
        },
        _ => panic!("Unexpected callback name")
    }

    Ok(instruction)
}


pub fn fill_custom_metas(
    metas: Vec<AccountMeta>,
) -> Vec<CustomAccountMeta> {
    let custom_metas = metas
        .iter()
        .map(|meta| {
            CustomAccountMeta{
                pubkey: meta.pubkey.to_string(),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            }
        })
        .collect();

    custom_metas
}

impl OrderBase {
    fn to_order(&self) -> Order {
        let maker_asset = Pubkey::from_str(&self.maker_asset).unwrap();
        let taker_asset = Pubkey::from_str(&self.taker_asset).unwrap();
        let maker = Pubkey::from_str(&self.maker).unwrap();

        Order {
            salt: self.salt,
            maker_asset,
            taker_asset,
            maker,
            making_amount: self.making_amount,
            taking_amount: self.taking_amount,
            predicate: self.predicate.clone(),
            callback: self.callback.clone(),
        }
    }
}