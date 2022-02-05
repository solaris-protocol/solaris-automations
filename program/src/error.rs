use thiserror::Error;
use solana_program::{
    program_error::{
        PrintProgramError, 
        ProgramError
    },
    decode_error::DecodeError,
    msg,
};
use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum AirdropError {
}

impl PrintProgramError for AirdropError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for AirdropError {
    fn type_of() -> &'static str {
        "Staking Error"
    }
}

impl From<AirdropError> for ProgramError {
    fn from(e: AirdropError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
