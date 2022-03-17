use solana_program::{
    account_info::{AccountInfo, next_account_info},
    pubkey::Pubkey,
    msg,
    entrypoint::ProgramResult,
    instruction::Instruction,

};
use arrayref::{array_refs, array_ref};
use uint::construct_uint;

use crate::error::SolarisAutoError;

// Pubkey is "FHdz7Ws3ettxHn8mJwD6PXLm7fMKZ91tMdicJoCR6fuk"
pub const PREDICATE_HEALTHFACTOR_ID: &[u8] = &[78, 11, 118, 213, 228, 92, 26, 55, 101, 204, 11, 75, 138, 91, 78, 249, 10, 197, 229, 133, 84, 247, 212, 213, 21, 232, 235, 119, 192, 110, 179, 177];

// U192 with 192 bits consisting of 3x64-bit words
construct_uint! {
    pub struct U192(3);
}

/// Large decimal values, precise to 18 digits
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Decimal(pub U192);

fn unpack_decimal(src: &[u8; 16]) -> Decimal {
   Decimal::from_scaled_val(u128::from_le_bytes(*src))
}

impl Decimal {
   /// Create decimal from scaled value
   pub fn from_scaled_val(scaled_val: u128) -> Self {
      Self(U192::from(scaled_val))
   }
}

const OBLIGATION_COLLATERAL_LEN: usize = 88; // 32 + 8 + 16 + 32
const OBLIGATION_LIQUIDITY_LEN: usize = 112; // 32 + 16 + 16 + 16 + 32
const MAX_OBLIGATION_RESERVES: usize = 10;
const OBLIGATION_LEN: usize = 1300; // 1 + 8 + 1 + 32 + 32 + 16 + 16 + 16 + 16 + 64 + 1 + 1 + (88 * 1) + (112 * 9)

/// Predicate that return Ok(()) if borrowed_value >= required_borrowed_value
/// 
/// Accounts required:
/// 
/// 0. `[]` Predicate healthfactor program id: 5kwKgdtbBN4HtGHtTuhDr37vJWAxTfx8QkxFGWwFqeoq
/// 1. `[]` Obligation account: https://pyth.network/developers/accounts/?cluster=devnet#
/// 
/// Instruction data format is 
/// ```
/// pub struct LendingHealthfactor {
///     required_borrowed_value: [u8; 16],
/// }
/// ```
pub fn process_healthfactor(
    instr: &Instruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let program_info = next_account_info(account_info_iter)?;
    let obligation_info = next_account_info(account_info_iter)?;
    
    let obligation = &mut *obligation_info.data.borrow_mut();
    let input = array_ref![obligation, 0, OBLIGATION_LEN];

    let (
        _version,
        _last_update_slot,
        _last_update_stale,
        _lending_market,
        _owner,
        _deposited_value,
        borrowed_value,
        allowed_borrow_value,
        unhealthy_borrow_value,
        _padding,
        _deposits_len,
        _borrows_len,
        _data_flat,
    ) = array_refs![
        &input,
        1,
        8,
        1,
        32,
        32,
        16,
        16,
        16,
        16,
        64,
        1,
        1,
        OBLIGATION_COLLATERAL_LEN + (OBLIGATION_LIQUIDITY_LEN * (MAX_OBLIGATION_RESERVES - 1))
    ];

    let required_borrowed_value = array_ref![instr.data, 0, 16];
    let required_borrowed_value = unpack_decimal(required_borrowed_value);
    let borrowed_value = unpack_decimal(&borrowed_value);

    msg!("borrowed value is {:?}", borrowed_value);
    msg!("required value is {:?}", required_borrowed_value);

    if borrowed_value >= required_borrowed_value {
        Ok(())
    } else {
        Err(SolarisAutoError::LendingHealthfactorFailed.into())
    }
}