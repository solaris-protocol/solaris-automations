use solana_program::{
    account_info::AccountInfo,
    instruction::Instruction,
    program::invoke,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program_error::ProgramError,
    msg,
};
use crate::error::SolarisAutoError;
use std::str::FromStr;

//Pubkey is "3Lf5PRfK3nibfrChcx2Hrh7g2WSgu3QBxLXSFY5WqMCA"
pub const HELPER_AND_ID: &[u8] = &[34, 192, 118, 35, 128, 32, 126, 54, 71, 146, 146, 47, 241, 227, 117, 146, 224, 12, 197, 13, 212, 150, 35, 113, 137, 30, 41, 185, 2, 214, 159, 231];
//Pubkey is "5kwKgdtbBN4HtGHtTuhDr37vJWAxTfx8QkxFGWwFqeoq"
pub const HELPER_OR_ID: &[u8] = &[70, 176, 37, 12, 106, 201, 74, 156, 64, 246, 254, 0, 195, 85, 90, 97, 88, 148, 195, 146, 24, 6, 246, 114, 86, 228, 185, 63, 193, 54, 105, 176];

pub fn check_predicate(
    inst: &[u8],
    accounts: &[AccountInfo],
) -> ProgramResult {
    // TODO: Find a way to avoid `bincode::deserialize` because
    //       it takes lots of CU.
    let predicate: Instruction = bincode::deserialize(inst)
        .or(Err(ProgramError::from(SolarisAutoError::IncorrectPredicateInst)))?;

    match predicate.program_id.as_ref() {
        HELPER_AND_ID => {
            process_and(&predicate.data, accounts)
        },
        // TODO: Find a way to create helper OR
        /*
        HELPER_OR_ID => {
            // process_or(&predicate.data, accounts)
        }*/
        _ => {
            invoke(&predicate, accounts)
        }
    }
}


pub fn process_and(
    encoded_instr: &[u8],
    accounts: &[AccountInfo],
) -> ProgramResult {
    let instructions: Vec<Instruction> = bincode::deserialize(encoded_instr)
        .or(Err(ProgramError::from(SolarisAutoError::IncorrectInstrAnd)))?;

    let instr_len = instructions.len();
    for i in 0..instr_len {
        invoke(&instructions[i], accounts)?;
    }

    Ok(())
}

pub fn process_or(
    encoded_instr: &[u8],
    accounts: &[AccountInfo],
) -> ProgramResult {
    let instructions: Vec<Instruction> = bincode::deserialize(encoded_instr)
        .or(Err(ProgramError::from(SolarisAutoError::IncorrectInstrOr)))?;
    
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