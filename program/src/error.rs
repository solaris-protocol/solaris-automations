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
    InvalidPredicateInst,
    #[error("Predicate AND failed")]
    PredicateAndFail,
    #[error("Cannot deserialize instructions for helper AND")]
    InvalidInstrAnd,
    #[error("Predicate OR failed")]
    PredicateOrFail,
    #[error("Cannot deserialize instructions for helper OR")]
    InvalidInstrOr,

    #[error("Cannot deserialize callback instruction")]
    InvalidCallbackInst,

    #[error("First instruction must be Ed25519 instruction")]
    InvalidInstrIndex,
    #[error("Program id for sign verify instruction must be Ed25519")]
    InvalidProgramIdSignVerify,
    #[error("Invalid instruction for Ed25519 program")]
    InvalidInstrSignVerify,
    #[error("Must be exactly 1 sign for order")]
    InvalidCountSignVerify,
    #[error("Signer must be maker")]
    InvalidSigner,
    #[error("Message size must be 32")]
    InvalidMsgSize,
    #[error("Invalid message for sign")]
    InvalidMsg,

    #[error("Cannot deserialize taker_asset_data instruction")]
    InvalidTakerAssetData,

    #[error("Invalid owner program_id")]
    InvalidOwnerProgramId,
    #[error("Oracle predicate failed")]
    OraclePredicateFailed,
    #[error("Lending healthfactor predicate failed")]
    LendingHealthfactorFailed,

    #[error("Order already closed")]
    OrderClosed,

    #[error("Data type missmatch")]
    DataTypeMissmatch,
    #[error("Order must be provided in first fill")]
    OrderIsNone,

    #[error("Math overflow")]
    MathOverflow,
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
