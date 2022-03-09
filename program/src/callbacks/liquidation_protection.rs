use solana_program::{
    account_info::{AccountInfo, next_account_info},
    instruction::{Instruction, AccountMeta},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
};

use byteorder::ByteOrder;

use crate::{
    error::SolarisAutoError,
    utils::get_seeds_delegate,
};

//Pubkey is "3Aix9sK31V2sSLwhbnBcYH6FtN79oTXDLN6vskjPsTbj"
pub const CALLBACK_SOLEND_LIQUIDATION_PROTECTION: &[u8] = &[32, 53, 9, 1, 169, 162, 247, 15, 108, 3, 155, 52, 149, 20, 2, 86, 154, 148, 207, 4, 134, 152, 207, 14, 16, 80, 168, 73, 173, 86, 193, 182];

/// Combines WithdrawObligationCollateral and RedeemReserveCollateral
///
/// Accounts required:
///
/// 0. `[]` Callback program
/// 1. `[writable]` Source withdraw reserve collateral supply SPL Token account.
/// 2. `[writable]` Destination collateral token account.
///                     Minted by withdraw reserve collateral mint.
/// 3. `[writable]` Withdraw reserve account - refreshed.
/// 4. `[writable]` Obligation account - refreshed.
/// 5. `[]` Lending market account.
/// 6. `[]` Derived lending market authority.
/// 7. `[writable]` User liquidity token account.
/// 8. `[writable]` Reserve collateral SPL Token mint.
/// 9. `[writable]` Reserve liquidity supply SPL Token account.
/// 10. `[signer]` Obligation owner
/// 11 `[signer]` User transfer authority ($authority).
/// 12. `[]` Clock sysvar.
/// 13. `[]` Token program id.
/// 14. `[]` Pyth price for reserve_account
/// 15. `[]` Switchboard price for reserve_account 
/// 
/// Instruction data format is 
/// ```
/// pub struct SolendLiquidationProtection {
///     solend_instruction_num: u8, // has value _15_ at this moment,
///     liquidity_amount: u64,
/// }
/// ```
pub fn process_callback_solend_liquidation_protection(
    instr: &Instruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let mut account_info_iter = &mut accounts.iter();

    let callback_program_info = next_account_info(account_info_iter)?;
    let solend_program_info = next_account_info(account_info_iter)?;

    let reserve_collateral_info = next_account_info(account_info_iter)?;
    let user_collateral_info = next_account_info(account_info_iter)?;
    let reserve_info = next_account_info(account_info_iter)?;
    let obligation_info = next_account_info(account_info_iter)?;
    let lending_market_info = next_account_info(account_info_iter)?;
    let lending_market_authority_info = next_account_info(account_info_iter)?;
    let user_liquidity_info = next_account_info(account_info_iter)?;
    let reserve_collateral_mint_info = next_account_info(account_info_iter)?;
    let reserve_liquidity_supply_info = next_account_info(account_info_iter)?;
    let obligation_owner_info = next_account_info(account_info_iter)?;
    let user_transfer_authority_info = next_account_info(account_info_iter)?;
    let clock = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let pyth_price_liquidity_info = next_account_info(account_info_iter)?; 
    let switchboard_price_liquidity_info = next_account_info(account_info_iter)?;

    let refresh_reserve = Instruction{
        program_id: *solend_program_info.key,
        accounts: vec![
            AccountMeta::new(*reserve_info.key, false),
            AccountMeta::new_readonly(*pyth_price_liquidity_info.key, false),
            AccountMeta::new_readonly(*switchboard_price_liquidity_info.key, false),
            AccountMeta::new_readonly(*clock.key, false),
        ],
        data: vec![3],
    };

    let refresh_obligation = Instruction{
        program_id: *solend_program_info.key,
        accounts: vec![
            AccountMeta::new(*obligation_info.key, false),
            AccountMeta::new_readonly(*clock.key, false),
            AccountMeta::new_readonly(*reserve_info.key, false),
            AccountMeta::new_readonly(*reserve_info.key, false),
        ],
        data: vec![7],
    };

    let solend_repay = Instruction {
        program_id: *solend_program_info.key,
        accounts: vec![
            AccountMeta::new(*reserve_collateral_info.key, false),
            AccountMeta::new(*user_collateral_info.key, false),
            AccountMeta::new(*reserve_info.key, false),
            AccountMeta::new(*obligation_info.key, false),
            AccountMeta::new_readonly(*lending_market_info.key, false),
            AccountMeta::new_readonly(*lending_market_authority_info.key, false),
            AccountMeta::new(*user_liquidity_info.key, false),
            AccountMeta::new(*reserve_collateral_mint_info.key, false),
            AccountMeta::new(*reserve_liquidity_supply_info.key, false),
            AccountMeta::new(*obligation_owner_info.key, true),
            AccountMeta::new(*user_transfer_authority_info.key, true),
            AccountMeta::new_readonly(*clock.key, false),
            AccountMeta::new_readonly(*token_program.key, false),
        ],
        data: instr.data.clone(),
    }; 

    invoke(
        &refresh_reserve,
        &accounts,
    )?;

    invoke(
        &refresh_obligation,
        &accounts,
    )?;

    invoke_signed(
        &solend_repay,
        &accounts,
        &[&get_seeds_delegate()],
    )?;

    Ok(())
}