use solana_program::{
    account_info::AccountInfo,
    instruction::Instruction,
    program::invoke,
    entrypoint::ProgramResult,
    program_error::{ProgramError, PrintProgramError},
};

use crate::{
    error::SolarisAutoError,
};

use super::liquidation_protection;

pub fn process_callback(
    instr: &[u8],
    accounts: &[AccountInfo],
) -> ProgramResult {
    bincode::deserialize::<Instruction>(instr)
        .or(Err(ProgramError::from(SolarisAutoError::InvalidCallbackInst)))
        .and_then(|callback| {
            match callback.program_id.as_ref() {
                liquidation_protection::CALLBACK_SOLEND_LIQUIDATION_PROTECTION => {
                    liquidation_protection::process_callback_solend_repay_obligation_liquidity(&callback, accounts)
                },
                _ => invoke(&callback, accounts)
            }
        })
}