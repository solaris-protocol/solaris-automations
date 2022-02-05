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
pub enum SolarisAutoError {
}

impl PrintProgramError for SolarisAutoError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for SolarisAutoError {
    fn type_of() -> &'static str {
        "Staking Error"
    }
}

impl From<SolarisAutoError> for ProgramError {
    fn from(e: SolarisAutoError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
