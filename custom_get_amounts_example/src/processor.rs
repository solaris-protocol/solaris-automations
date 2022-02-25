use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed, set_return_data},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use borsh::{
    BorshDeserialize,
};

use spl_token::state::Account as TokenAccount;

use crate::{error::CustomGetAmountsError, instruction::CustomGetAmountsInstruction};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = CustomGetAmountsInstruction::try_from_slice(instruction_data)?;

        match instruction {
            CustomGetAmountsInstruction::GetMakerAmount
            => {
                Self::process_get_maker_amount(
                    program_id,
                    accounts,
                )
            },
            CustomGetAmountsInstruction::GetTakerAmount 
            => {
                Self::process_get_taker_amount(
                    program_id,
                    accounts,
                )
            }
        }
    }

    pub fn process_get_maker_amount(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let maker_amount: u64 = 1_000_000_000;

        set_return_data(&maker_amount.to_le_bytes());

        Ok(())
    }

    pub fn process_get_taker_amount(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        Ok(())
    }
}
