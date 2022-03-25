pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod helpers;
pub mod callbacks;
pub mod verify_sign;
pub mod utils;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("HX8vWLXQoVt4EsxMYUq7BHy1Bg9TRfc5YeRo4oqBnipP");
