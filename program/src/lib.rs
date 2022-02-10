pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod helpers;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("EyJ4ZNzAK8HJJrRbTTE6x769RA2h95zj826194DxyEbw");
