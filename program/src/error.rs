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

#[derive(Error, FromPrimitive, Debug, Copy, Clone)]
pub enum SolarisAutoError {
    #[error("Predicate failed")]
    PredicateFail,
    #[error("Cannot deserialize predicate instruction")]
    IncorrectPredicateInst,
    #[error("Predicate AND failed")]
    PredicateAndFail,
    #[error("Cannot deserialize instructions for helper AND")]
    IncorrectInstrAnd,
    #[error("Predicate OR failed")]
    PredicateOrFail,
    #[error("Cannot deserialize instructions for helper OR")]
    IncorrectInstrOr,
}

impl PrintProgramError for SolarisAutoError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for SolarisAutoError {
    fn type_of() -> &'static str {
        "Solaris Automations Error"
    }
}

impl From<SolarisAutoError> for ProgramError {
    fn from(e: SolarisAutoError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
