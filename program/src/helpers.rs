use solana_program::{
    account_info::AccountInfo,
    instruction::Instruction,
    program::invoke,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};
use std::str::FromStr;

//Pubkey is "3Lf5PRfK3nibfrChcx2Hrh7g2WSgu3QBxLXSFY5WqMCA"
pub const HELPER_ADD_ID: &[u8] = &[34, 192, 118, 35, 128, 32, 126, 54, 71, 146, 146, 47, 241, 227, 117, 146, 224, 12, 197, 13, 212, 150, 35, 113, 137, 30, 41, 185, 2, 214, 159, 231];
//Pubkey is "5kwKgdtbBN4HtGHtTuhDr37vJWAxTfx8QkxFGWwFqeoq"
pub const HELPER_OR_ID: &[u8] = &[70, 176, 37, 12, 106, 201, 74, 156, 64, 246, 254, 0, 195, 85, 90, 97, 88, 148, 195, 146, 24, 6, 246, 114, 86, 228, 185, 63, 193, 54, 105, 176];

pub fn check_predicate(
    inst: &[u8],
    accounts: &[AccountInfo],
) -> ProgramResult {
    // TODO: Find a way to avoid `bincode::deserialize` because
    //       it takes much CU.
    let predicate: Instruction = 
            bincode::deserialize(inst)
                .expect("Cannot deserialize instruction");

    match predicate.program_id.as_ref() {
        HELPER_ADD_ID => {
            process_and(&predicate.data)
        },
        _ => {
            invoke(&predicate,accounts)
        }
    }
}


pub fn process_and(
    encoded_instructions: &[u8],
) -> ProgramResult {
    let instructions: Vec<Instruction> = 
        bincode::deserialize(encoded_instructions)
            .expect("Cannot deserialize instructions for AND");

    Ok(())
}