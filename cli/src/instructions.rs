use solana_sdk::{
    pubkey::Pubkey,
    instruction::{Instruction, AccountMeta},
    sysvar,
    system_program,
};
use spl_token;
use borsh::{
    BorshSerialize,     
    BorshDeserialize,
    BorshSchema,
};

#[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
pub struct Order {
    pub salt: u64,
    pub maker_asset: Pubkey,
    pub taker_asset: Pubkey,
    pub maker: Pubkey,
    pub receiver: Pubkey,
    pub allowed_sender: Pubkey,
    pub making_amount: u64,
    pub taking_amount: u64,
    pub get_maker_amount: Vec<u8>,
    pub get_taker_amount: Vec<u8>,
    pub predicate: Vec<u8>,
    pub interaction: Vec<u8>,
}

#[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
pub struct FillOrderArgs {
    pub order: Order,
    pub making_amount: u64,
    pub taking_amount: u64,
    pub threshold_amount: u64,
    pub predicate_infos_count: u8,
}

#[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
pub enum SolarisAutoInstruction {
    /// 0
    /// Fill order
    /// 
    /// Accounts expected:
    /// 
    /// 0. `[writable]` Maker account
    /// 1. `[writable]` Taker account 
    /// 2. `[]` PDA delegate
    /// 3. `[]` Sysvar instructions
    /// 4. `[]` Spl-token 
    /// *TODO* 5. `[writable]` PDA order. Seeds: []  
    /// 6.. Accounts that required by predicate instruction 
    FillOrder(FillOrderArgs),
    ///
    /// 1
    /// Init PDA delegate. Account which must be approved for transfer 
    /// tokens from maker token-account.
    /// 
    /// Accounts expected:
    /// 
    /// 0. `[signer]` Payer
    /// 1. `[writable]` PDA delegate. Seeds: ["solaris-automations", "delegate", bump]
    /// 2. `[]` system-program
    InitDelegate,
}

pub fn fill_order(
    program_id: &Pubkey,
    maker: &Pubkey,
    taker: &Pubkey,
    delegate: &Pubkey,
    predicate_accounts: &[Pubkey],
    maker_asset_data_accounts: &[Pubkey],
    taker_asset_data_accounts: &[Pubkey],

    order: Order,
    making_amount: u64,
    taking_amount: u64,
    threshold_amount: u64,
) -> Instruction {
    let fill_order_args = FillOrderArgs {
        order: order,
        making_amount,
        taking_amount,
        threshold_amount,
        predicate_infos_count: predicate_accounts.len() as u8,
    };

    let data = SolarisAutoInstruction::FillOrder(fill_order_args)
        .try_to_vec().unwrap();

    let mut accounts = vec![
        AccountMeta::new(*maker, false),
        AccountMeta::new(*taker, true),
        AccountMeta::new(*delegate, false),
        AccountMeta::new_readonly(sysvar::instructions::id(), false),    
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    predicate_accounts.iter()
        .for_each(|id| accounts.push(AccountMeta::new_readonly(*id, false)));
    maker_asset_data_accounts.iter()
        .for_each(|id| accounts.push(AccountMeta::new(*id, false)));
    taker_asset_data_accounts.iter()
        .for_each(|id| accounts.push(AccountMeta::new(*id, false)));

    Instruction{
        program_id: *program_id,
        accounts,
        data,
    }
}

pub fn init_delegate(
    program_id: &Pubkey,
    payer: &Pubkey,
    delegate: &Pubkey,
) -> Instruction {
    let data = SolarisAutoInstruction::InitDelegate
        .try_to_vec().unwrap();

    let accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(*delegate, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction{
        program_id: *program_id,
        accounts,
        data,
    }
}