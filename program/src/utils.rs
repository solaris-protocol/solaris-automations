use solana_program::{
    entrypoint::ProgramResult,
    program_error::ProgramError,
    account_info::AccountInfo,
    pubkey::Pubkey,
};

use crate::{
    error::SolarisAutoError,
    state::{PREFIX, DELEGATE, BUMP_DELEGATE},
};

pub fn get_seeds_delegate() -> [&'static [u8]; 3] {
    [PREFIX.as_bytes(), DELEGATE.as_bytes(), &[BUMP_DELEGATE]]
}

pub fn assert_owned_by(
    info: &AccountInfo,
    owner_id: &Pubkey
) -> ProgramResult {
    if *info.owner != *owner_id {
        Err(SolarisAutoError::InvalidOwnerProgramId.into())
    } else {
        Ok(())
    }
}