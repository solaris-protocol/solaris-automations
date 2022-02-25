use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, 
    pubkey::Pubkey, program_error::PrintProgramError,
};

use crate::{processor::Processor, error::CustomGetAmountsError};

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(err) = Processor::process(program_id, accounts, instruction_data) {
        err.print::<CustomGetAmountsError>();
        return Err(err);
    }

    Ok(())
}
