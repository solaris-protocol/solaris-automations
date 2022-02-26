use serde::{Serialize, Deserialize};
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
    pub callback: Vec<u8>,
}

#[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
#[repr(C)]
pub struct FillOrderArgs {
    pub order: Option<Order>,
    pub making_amount: u64,
    pub taking_amount: u64,
    pub threshold_amount: u64,
    pub get_maker_amount_infos_num: u8,
    pub get_taker_amount_infos_num: u8, 
    pub predicate_infos_num: u8,
    pub callback_infos_num: u8,
}

#[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
pub enum SolarisAutoInstruction {
    /// 0
    /// Fill order
    /// 
    /// Accounts expected:
    /// 
    /// 0. `[]` Maker account
    /// 1. `[signer, writable]` Taker account 
    /// 3. `[]` Sysvar instructions
    /// 4. `[writable]` Onchain order. Seeds: [prefix, onchain_order, order_hash]
    /// 5. `[]` system-program
    /// 6.. Accounts that required by get_maker_amount instruction  
    ///  .. Accounts that required by predicate instruction
    /// 
    /// OrderStage == Filled. Callback and transfers
    /// 
    /// .. `[writable]` Taker token-account
    /// .. `[writable]` Maker token-account
    /// .. `[]` delegate
    /// .. `[]` spl-token
    FillOrder(FillOrderArgs),
    ///
    /// 1
    /// 
    /// Accounts expected:
    /// 
    /// 0. `[writable]` Source liquidity token account.
    ///                     $authority can transfer $liquidity_amount.
    /// 1. `[writable]` Destination collateral token account.
    /// 2. `[writable]` Reserve account.
    /// 3. `[writable]` Reserve liquidity supply SPL Token account.
    /// 4. `[writable]` Reserve collateral SPL Token mint.
    /// 5. `[]` Lending market account.
    /// 6. `[]` Derived lending market authority.
    /// 7. `[writable]` Destination deposit reserve collateral supply SPL Token account.
    /// 8. `[writable]` Obligation account.
    /// 9. `[signer]` Obligation owner.
    /// 10 `[]` Pyth price oracle account.
    /// 11 `[]` Switchboard price feed oracle account.
    /// 12 `[signer]` User transfer authority ($authority).
    /// 13 `[]` Clock sysvar.
    /// 14 `[]` Token program id.
    /// 15 `[]` Solend program
    ProxyDepositReserveLiquidityAndObligationCollateral {
        liquidity_amount: u64,
    },
    /// 
    /// 2
    /// Init PDA delegate. Account which must be approved for transfer 
    /// tokens from maker token-account.
    /// 
    /// Accounts expected:
    /// 
    /// 0. `[signer]` Payer
    /// 1. `[writable]` PDA delegate. Seeds: ["solaris-automations", "delegate", bump]
    /// 2. `[]` system-program
    InitDelegate,
    ///
    /// 3
    ///
    /// Accounts expected:
    /// 
    /// 0. `[signer]` Contract owner
    /// 1. `[]` PDA delegate. Seeds: ["solaris-automations", "delegate", bump]
    /// 2. `[]` Solend program
    /// 3. `[writable]` Obligation account for delegate
    /// 4. `[]` Lending market info
    /// 5. `[writable]` Collateral token account info. Seeds: ["solaris-automations, "collateral_ta", bump]
    /// 6. `[]` Collateral mint info
    /// 7. `[]` System-program 
    /// 8. `[]` Spl-token
    InitSolendAccountsForDelegate,
}

pub enum OrderStage {
    Create,
    Filled,
    Closed,
}

pub fn fill_order(
    program_id: &Pubkey,
    maker: &Pubkey,
    taker: &Pubkey,
    onchain_order: &Pubkey, 
    delegate: &Pubkey,
    get_maker_amount_accounts: &[Pubkey],
    get_taker_amount_accounts: &[Pubkey],
    predicate_accounts: &[Pubkey],
    callback_accounts: &[Pubkey],
    taker_ta_taker_asset_account: &Pubkey,
    maker_ta_taker_asset_account: &Pubkey,
    taker_ta_maker_asset_account: &Pubkey,
    maker_ta_maker_asset_account: &Pubkey,

    order: Option<Order>,
    making_amount: u64,
    taking_amount: u64,
    threshold_amount: u64,

    order_stage: OrderStage,
) -> Instruction {
    let fill_order_args = FillOrderArgs {
        order,
        making_amount,
        taking_amount,
        threshold_amount,
        get_maker_amount_infos_num: get_maker_amount_accounts.len() as u8,
        get_taker_amount_infos_num: get_taker_amount_accounts.len() as u8,
        predicate_infos_num: predicate_accounts.len() as u8,
        callback_infos_num: callback_accounts.len() as u8,   
    };

    let data = SolarisAutoInstruction::FillOrder(fill_order_args)
        .try_to_vec().unwrap();

    let mut accounts = vec![
        AccountMeta::new_readonly(*maker, false),
        AccountMeta::new(*taker, true),
        AccountMeta::new_readonly(sysvar::instructions::id(), false),    
        AccountMeta::new(*onchain_order, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    if !get_maker_amount_accounts.is_empty(){
        accounts.push(AccountMeta::new_readonly(get_maker_amount_accounts[0], false));
        get_maker_amount_accounts[1..].iter()
            .for_each(|id| accounts.push(AccountMeta::new(*id, false)));
    }

    if !get_taker_amount_accounts.is_empty() {
        accounts.push(AccountMeta::new_readonly(get_taker_amount_accounts[0], false));
        get_taker_amount_accounts[1..].iter()
            .for_each(|id| accounts.push(AccountMeta::new(*id, false)));
    }

    if !predicate_accounts.is_empty() {
        accounts.push(AccountMeta::new_readonly(predicate_accounts[0], false));
        predicate_accounts[1..].iter()
            .for_each(|id| accounts.push(AccountMeta::new(*id, false)));
    }

    if !callback_accounts.is_empty() {
        accounts.push(AccountMeta::new_readonly(callback_accounts[0], false));
        callback_accounts[1..].iter()
            .for_each(|id| accounts.push(AccountMeta::new(*id, false)));
    }

    match order_stage {
        OrderStage::Create => {

        },
        OrderStage::Filled => {
            accounts.push(AccountMeta::new(*taker_ta_taker_asset_account, false));
            accounts.push(AccountMeta::new(*maker_ta_taker_asset_account, false));
            accounts.push(AccountMeta::new(*maker_ta_maker_asset_account, false));
            accounts.push(AccountMeta::new(*taker_ta_maker_asset_account, false));
            accounts.push(AccountMeta::new(*delegate, false));
            accounts.push(AccountMeta::new_readonly(spl_token::ID, false));
        }, 
        OrderStage::Closed => {

        }
    }

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