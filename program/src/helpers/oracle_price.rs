use solana_program::{
    account_info::{AccountInfo, next_account_info},
    instruction::Instruction,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
};
use std::str::FromStr; // Will be deleted
use std::convert::TryInto;
use byteorder::ByteOrder;

use chainlink_solana; 
use pyth_client;

use crate::{
    utils::assert_owned_by,
    error::SolarisAutoError,
};

//Pubkey is "5kwKgdtbBN4HtGHtTuhDr37vJWAxTfx8QkxFGWwFqeoq"
pub const HELPER_PYTH_ID: &[u8] = &[70, 176, 37, 12, 106, 201, 74, 156, 64, 246, 254, 0, 195, 85, 90, 97, 88, 148, 195, 146, 24, 6, 246, 114, 86, 228, 185, 63, 193, 54, 105, 176];

/// Predicate that return Ok(()) if price on Pyth data feed
/// [more/less] than required amount
/// 
/// Accounts required:
/// 
/// 0. `[]` Helper pyth program id: 5kwKgdtbBN4HtGHtTuhDr37vJWAxTfx8QkxFGWwFqeoq
/// 1. `[]` Pyth price account: https://pyth.network/developers/accounts/?cluster=devnet#
/// 
/// 
/// Instruction data format is 
/// ```
/// pub struct HelperPythPrice {
///     amount: u64,
///     less_than_pyth_price: bool,
/// }
/// ```
pub fn process_pyth_price(
    instr: &Instruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let program_info = next_account_info(account_info_iter)?;
    let pyth_price_info = next_account_info(account_info_iter)?;
    assert_owned_by(pyth_price_info, &Pubkey::from_str("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s").unwrap())?;
    // assert_owned_by(pyth_price_info, pyth_client::ID)?;

    let pyth_price = pyth_client::load_price(&pyth_price_info.data.borrow_mut())
        .and_then(|price| {
            let price_conf = price.get_current_price().unwrap();

            Ok(price_conf)
        })?;

    msg!("price_conf is {:?}", pyth_price);

    let amount = byteorder::LE::read_u64(&instr.data[0..8]);
    let less_than_pyth_price = instr.data[8] != 0;

    // TODO: comparison with conf
    match less_than_pyth_price {
        true => {
            if amount >= pyth_price.price.try_into().unwrap() {
                return Err(SolarisAutoError::OraclePredicateFailed.into())
            }
        },
        false => {
            if amount < pyth_price.price.try_into().unwrap() {
                return Err(SolarisAutoError::OraclePredicateFailed.into())
            }
        }
    }

    Ok(())
}