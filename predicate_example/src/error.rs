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
pub enum PredicateError {
}

impl PrintProgramError for PredicateError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for PredicateError {
    fn type_of() -> &'static str {
        "Predicate Error"
    }
}

impl From<PredicateError> for ProgramError {
    fn from(e: PredicateError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
