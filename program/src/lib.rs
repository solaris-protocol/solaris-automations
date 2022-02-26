pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod helpers;
pub mod verify_sign;
pub mod utils;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("mNMPYGkwTeSaNd8VJJBya989vBtim5ccnodVV6mq9Qg");
