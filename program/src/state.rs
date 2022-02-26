use borsh::{
    BorshSerialize,     
    BorshDeserialize,
    BorshSchema,
};
use solana_program::{
    program_error::ProgramError,
    account_info::AccountInfo,
};

pub const PREFIX: &str = "solaris-automations";
pub const ONCHAIN_ORDER: &str = "onchain_order";
pub const DELEGATE: &str = "delegate";
pub const COLLATERAL_TA: &str = "collateral_ta";

pub const BUMP_DELEGATE: u8 = 255;
pub const BUMP_COLLATERAL_TA: u8 = 255;

pub const ONCHAIN_ORDER_STATE_SIZE: usize = 58; 

#[derive(BorshSchema, BorshDeserialize, BorshSerialize)]
pub enum Key {
    OnchainOrder,
}

#[derive(BorshSchema, BorshDeserialize, BorshSerialize)]
pub enum OrderStage {
    Create,
    Filled,
    Closed,
}

#[derive(BorshSchema, BorshDeserialize, BorshSerialize)]
pub struct OnchainOrder {
    pub key: Key,
    pub order_hash: [u8; 32],
    pub making_amount: u64,
    pub taking_amount: u64,
    pub remaining_maker_amount: u64,
    pub get_maker_amount: Vec<u8>,
    pub get_taker_amount: Vec<u8>,
    pub predicate: Vec<u8>,
    pub callback: Vec<u8>,
    pub stage: OrderStage,
}

impl OnchainOrder {
    pub fn from_account_info(a: &AccountInfo) -> Result<OnchainOrder, ProgramError> {
        let onchain_order = OnchainOrder::try_from_slice(
            &a.data.borrow_mut(),
        )?;

        Ok(onchain_order)
    }
}