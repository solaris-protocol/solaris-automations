use borsh::{BorshSerialize, BorshDeserialize, BorshSchema};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    instruction::Instruction,
    program::invoke,
    entrypoint::ProgramResult,
    program_error::{ProgramError, PrintProgramError},
    pubkey::Pubkey,
    msg,
};
use byteorder::ByteOrder;
use std::{str::FromStr, convert::TryInto};
use chainlink_solana; 
use pyth_client;

use crate::{
    error::SolarisAutoError,
    utils::{
        assert_owned_by,
    }
};

//Pubkey is "3Lf5PRfK3nibfrChcx2Hrh7g2WSgu3QBxLXSFY5WqMCA"
pub const HELPER_AND_ID: &[u8] = &[34, 192, 118, 35, 128, 32, 126, 54, 71, 146, 146, 47, 241, 227, 117, 146, 224, 12, 197, 13, 212, 150, 35, 113, 137, 30, 41, 185, 2, 214, 159, 231];
//Pubkey is "5kwKgdtbBN4HtGHtTuhDr37vJWAxTfx8QkxFGWwFqeoq"
pub const HELPER_PYTH_ID: &[u8] = &[70, 176, 37, 12, 106, 201, 74, 156, 64, 246, 254, 0, 195, 85, 90, 97, 88, 148, 195, 146, 24, 6, 246, 114, 86, 228, 185, 63, 193, 54, 105, 176];

pub fn check_predicate(
    inst: &[u8],
    accounts: &[AccountInfo],
) -> ProgramResult {
    _check_predicate(inst, accounts, invoke_predicate)
}

fn _check_predicate<F>(
    inst: &[u8],
    accounts: &[AccountInfo],
    invoke_predicate: F
) -> ProgramResult 
    where F: Fn(&Instruction, &[AccountInfo]) -> ProgramResult {
    // TODO: Find a way to avoid `bincode::deserialize` because
    //       it takes lots of CU.
    bincode::deserialize::<Instruction>(inst)
        .or(Err(ProgramError::from(SolarisAutoError::InvalidPredicateInst)))
        .and_then(|predicate| {
            match predicate.program_id.as_ref() {
                HELPER_AND_ID => {
                    process_and(&predicate.data, accounts, invoke_predicate)
                },
                // TODO: Find a way to create helper OR
                /*
                HELPER_OR_ID => {
                    // process_or(&predicate.data, accounts)
                }*/
                _ => {
                    invoke_predicate(&predicate, accounts)
                }
            }
        })
}

fn invoke_predicate(
    instr: &Instruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    match instr.program_id.as_ref() {
        HELPER_PYTH_ID => {
            process_pyth_price(instr, accounts)
        },
        _ => invoke(instr, accounts)
    }
}

fn process_and<F>(
    encoded_instr: &[u8],
    accounts: &[AccountInfo],
    invoke_predicate: F,
) -> ProgramResult 
    where F: Fn(&Instruction, &[AccountInfo]) -> ProgramResult {
    let instructions: Vec<Instruction> = bincode::deserialize(encoded_instr)
        .or(Err(ProgramError::from(SolarisAutoError::InvalidInstrAnd)))?;
    
    let instr_len = instructions.len();
    for i in 0..instr_len {
        if let Err(error) = invoke_predicate(&instructions[i], accounts) {
            error.print::<SolarisAutoError>();
            return Err(ProgramError::from(SolarisAutoError::PredicateAndFail))
        }
    }

    Ok(())
}

fn process_or(
    encoded_instr: &[u8],
    accounts: &[AccountInfo],
) -> ProgramResult {
    let instructions: Vec<Instruction> = bincode::deserialize(encoded_instr)
        .or(Err(ProgramError::from(SolarisAutoError::InvalidInstrOr)))?;
    
    let instr_len = instructions.len();
    for i in 0..instr_len {
        // Invoke will terminate program execution if 
        // return value is Err(_)
        if let Ok(_) = invoke(&instructions[i], accounts) {
            return Ok(())
        }
    }

    Err(SolarisAutoError::PredicateOrFail.into())
}

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
fn process_pyth_price(
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

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::{
        pubkey::Pubkey,
        system_program,
        instruction::{Instruction, AccountMeta},
    };

    #[test]
    fn check_predicate() {
        let some_pubkey = Pubkey::new_unique();

        let instruction = bincode::serialize(&Instruction{
            program_id: Pubkey::new_unique(),
            accounts: vec![AccountMeta::new(some_pubkey, false)],
            data: vec![],
        }).unwrap();

        let invoke_plug_ok = |_instr: &Instruction, _infos: &[AccountInfo]| Ok(());
        let invoke_plug_err = |_instr: &Instruction, _infos: &[AccountInfo]| Err(ProgramError::Custom(0));

        assert_eq!(
            _check_predicate(&instruction, &[], invoke_plug_ok),
            Ok(())
        );

        assert_eq!(
            _check_predicate(&instruction, &[], invoke_plug_err),
            Err(ProgramError::Custom(0)),
        );
    }

    #[test]
    fn check_predicate_and() {
        let some_pubkey = Pubkey::new_unique();

        let instructions_and = bincode::serialize(&vec![
            Instruction{
                program_id: Pubkey::new_unique(), 
                accounts: vec![],
                data: vec![],
            },
            Instruction{
                program_id: Pubkey::new_unique(),
                accounts: vec![],
                data: vec![],
            }
        ]).unwrap();

        let instruction = bincode::serialize(&Instruction{
            program_id: Pubkey::new(HELPER_AND_ID),
            accounts: vec![AccountMeta::new(some_pubkey, false)],
            data: instructions_and,
        }).unwrap();

        let invoke_plug_ok = |_instr: &Instruction, _infos: &[AccountInfo]| Ok(());
        let invoke_plug_err = |_instr: &Instruction, _infos: &[AccountInfo]| Err(ProgramError::Custom(0));

        assert_eq!(
            _check_predicate(&instruction, &[], invoke_plug_ok),
            Ok(())
        );

        assert_eq!(
            _check_predicate(&instruction, &[], invoke_plug_err),
            Err(ProgramError::from(SolarisAutoError::PredicateAndFail)),
        );
    }
}