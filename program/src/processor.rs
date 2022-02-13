use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
    sysvar,
    keccak,
};
use borsh::{
    BorshDeserialize,
};

use crate::{
    instruction::SolarisAutoInstruction,
    helpers::check_predicate,
    verify_sign::is_valid_signature,
};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = SolarisAutoInstruction::try_from_slice(instruction_data)?;

        match instruction {
            SolarisAutoInstruction::FillOrder {
                predicate
            } => {
                msg!("Instruction: FillOrder");
                Self::process_fill_order(program_id, accounts, predicate)
            }
        }
    }

    pub fn process_fill_order(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        predicate: Vec<u8>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let maker = next_account_info(account_info_iter)?;
        let taker = next_account_info(account_info_iter)?;
        let sysvar_instr = next_account_info(account_info_iter)?;

        let predicate_infos: Vec<AccountInfo> = 
            account_info_iter
                .map(|account_info| account_info )
                .cloned()
                .collect();

        let order_hash = keccak::hash(&predicate);
        is_valid_signature(maker.key, order_hash.as_ref(), sysvar_instr)?;

        check_predicate(&predicate, &predicate_infos[..])?;

        Ok(())
    }
}
