use thiserror::Error;
use num_derive::FromPrimitive;
use solana_program::{
    program_error::{
        PrintProgramError, 
        ProgramError
    },
    decode_error::DecodeError,
    msg,
};

#[derive(Error, Debug, Copy, Clone, FromPrimitive)]
pub enum CustomGetAmountsError {
}

impl PrintProgramError for CustomGetAmountsError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for CustomGetAmountsError {
    fn type_of() -> &'static str {
        "Predicate Error"
    }
}

impl From<CustomGetAmountsError> for ProgramError {
    fn from(e: CustomGetAmountsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
