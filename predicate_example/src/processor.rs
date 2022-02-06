use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

use spl_token::state::Account as TokenAccount;

use crate::{error::PredicateError, instruction::PredicateInstruction};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = PredicateInstruction::try_from_slice(instruction_data)?;

        match instruction {
            PredicateInstruction::IsHFBelow {
                return_val,
            }
            => {
                Self::process_is_hf_below(
                    program_id,
                    accounts,
                    return_val,
                )
            }
        }
    }

    pub fn process_is_hf_below(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        return_val: bool,
    ) -> ProgramResult {
        match return_val {
            true => Ok(()),
            false => return Err(ProgramError::Custom(0)),
        }
    }
}
