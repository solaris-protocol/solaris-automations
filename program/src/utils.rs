use solana_program::{
    entrypoint::ProgramResult,
    program_error::ProgramError,
    account_info::AccountInfo,
    pubkey::Pubkey, 
    instruction::{Instruction, AccountMeta},
    system_instruction,
    rent::Rent,
    clock::Clock,
    sysvar::{SysvarId, Sysvar},    
    program_pack::Pack,                                                     
};
use spl_token::state::Account as TokenAccount;
use borsh::{
    BorshSchema,
    BorshDeserialize,
    BorshSerialize,
};

use crate::{
    id as program_id,
    instruction::Order,
    error::SolarisAutoError,
    state::{
        Key,
        PREFIX, ONCHAIN_ORDER, DELEGATE, 
        BUMP_DELEGATE, 
        ONCHAIN_ORDER_STATE_SIZE, COLLATERAL_TA, BUMP_COLLATERAL_TA
    },
};

pub fn get_seeds_delegate() -> [&'static [u8]; 3] {
    [PREFIX.as_bytes(), DELEGATE.as_bytes(), &[BUMP_DELEGATE]]
}

pub fn get_seeds_collateral_ta() -> [&'static [u8]; 3] {
    [PREFIX.as_bytes(), COLLATERAL_TA.as_bytes(), &[BUMP_COLLATERAL_TA]]
}

pub fn get_bump_onchain_order(order_hash: &[u8]) -> u8 {
    let (_, bump) = Pubkey::find_program_address(
        &[PREFIX.as_bytes(), ONCHAIN_ORDER.as_bytes(), order_hash],
        &program_id(),
    );

    bump
} 

pub fn assert_owned_by(
    info: &AccountInfo,
    owner_id: &Pubkey
) -> ProgramResult {
    if *info.owner != *owner_id {
        Err(SolarisAutoError::InvalidOwnerProgramId.into())
    } else {
        Ok(())
    }
}

pub fn create_onchain_order(
    from_id: &Pubkey,
    onchain_order_id: &Pubkey,
    order: &Order,
) -> Result<Instruction, ProgramError> {
    let rent = Rent::get()?;
    let size = ONCHAIN_ORDER_STATE_SIZE + 
        4 + order.get_maker_amount.len() +
        4 + order.get_taker_amount.len() +
        4 + order.predicate.len() +
        4 + order.callback.len();

    let min_rent_exempt = rent.minimum_balance(size);

    Ok(system_instruction::create_account(
        from_id,
        onchain_order_id,
        min_rent_exempt,
        size as u64,
        &program_id(),
    ))
}

pub fn create_collateral_token_account(
    from_id: &Pubkey,
    collateral_token_account_id: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let rent = Rent::get()?;
    let size = TokenAccount::LEN;

    let min_rent_exempt = rent.minimum_balance(size);

    Ok(system_instruction::create_account(
        from_id,
        collateral_token_account_id,
        min_rent_exempt,
        size as u64,
        &spl_token::ID,
    ))
}

pub fn solend_init_obligation(
    solend_program_id: &Pubkey,
    obligation_account_id: &Pubkey,
    lending_market_id: &Pubkey,
    obligation_owner_id: &Pubkey,
) -> Instruction {
    Instruction{
        program_id: *solend_program_id,
        accounts: vec![
            AccountMeta::new(*obligation_account_id, false),
            AccountMeta::new_readonly(*lending_market_id, false),
            AccountMeta::new_readonly(*obligation_owner_id, true),
            AccountMeta::new_readonly(Clock::id(), false),
            AccountMeta::new_readonly(Rent::id(), false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: vec![6],
    }
}