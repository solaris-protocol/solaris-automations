use solana_program::{
    account_info::AccountInfo,
    instruction::Instruction,
    program::invoke,
    entrypoint::ProgramResult,
    program_error::ProgramError,
};
use crate::error::SolarisAutoError;

//Pubkey is "3Lf5PRfK3nibfrChcx2Hrh7g2WSgu3QBxLXSFY5WqMCA"
pub const HELPER_AND_ID: &[u8] = &[34, 192, 118, 35, 128, 32, 126, 54, 71, 146, 146, 47, 241, 227, 117, 146, 224, 12, 197, 13, 212, 150, 35, 113, 137, 30, 41, 185, 2, 214, 159, 231];
//Pubkey is "5kwKgdtbBN4HtGHtTuhDr37vJWAxTfx8QkxFGWwFqeoq"
//pub const HELPER_OR_ID: &[u8] = &[70, 176, 37, 12, 106, 201, 74, 156, 64, 246, 254, 0, 195, 85, 90, 97, 88, 148, 195, 146, 24, 6, 246, 114, 86, 228, 185, 63, 193, 54, 105, 176];

pub fn check_predicate(
    inst: &[u8],
    accounts: &[AccountInfo],
) -> ProgramResult {
    _check_predicate(inst, accounts, invoke)
}

fn _check_predicate<F>(
    inst: &[u8],
    accounts: &[AccountInfo],
    invoke: F
) -> ProgramResult 
    where F: Fn(&Instruction, &[AccountInfo]) -> ProgramResult {
    // TODO: Find a way to avoid `bincode::deserialize` because
    //       it takes lots of CU.
    bincode::deserialize::<Instruction>(inst)
        .or(Err(ProgramError::from(SolarisAutoError::InvalidPredicateInst)))
        .and_then(|predicate| {
            match predicate.program_id.as_ref() {
                HELPER_AND_ID => {
                    process_and(&predicate.data, accounts, invoke)
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
        })
}

fn process_and<F>(
    encoded_instr: &[u8],
    accounts: &[AccountInfo],
    invoke: F,
) -> ProgramResult 
    where F: Fn(&Instruction, &[AccountInfo]) -> ProgramResult {
    let instructions: Vec<Instruction> = bincode::deserialize(encoded_instr)
        .or(Err(ProgramError::from(SolarisAutoError::InvalidInstrAnd)))?;
    
    let instr_len = instructions.len();
    for i in 0..instr_len {
        if let Err(_) = invoke(&instructions[i], accounts) {
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