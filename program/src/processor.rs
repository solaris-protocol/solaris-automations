use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError, 
    program::{invoke, invoke_signed}, 
    pubkey::Pubkey,
    system_instruction,
    msg,
    keccak,
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};

use crate::{
    helpers::{
        predicate_helpers::check_predicate,
        //get_amounts::process_get_amounts,
        get_amounts::{get_maker_amount, get_taker_amount}, 
    },
    verify_sign::is_valid_signature,
    error::SolarisAutoError,
    instruction::{
        SolarisAutoInstruction,
        FillOrderArgs,
    },
    state::{
        Key,
        OnchainOrder,
        OrderStage,
        PREFIX,
        ONCHAIN_ORDER,
    },
    utils::{
        get_seeds_delegate,
        get_bump_onchain_order,
        create_onchain_order,
    },
};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = SolarisAutoInstruction::try_from_slice(instruction_data)?;

        match instruction {
            SolarisAutoInstruction::FillOrder(args) 
            => {
                msg!("Instruction: FillOrder");
                Self::process_fill_order(program_id, accounts, args)
            },
            SolarisAutoInstruction::ProxyDepositReserveLiquidityAndObligationCollateral {
                liquidity_amount,
            } => {
                msg!("Instruction: ProxyDepositReserveLiquidityAndObligationCollateral");
                Self::process_proxy_deposit_reserve_liquidity_and_obligation_collateral(program_id, accounts, args)
            }
            SolarisAutoInstruction::InitDelegate
            => {
                msg!("Instruction: InitDelegate");
                Self::process_init_delegate(program_id, accounts)
            }
        }
    }

    pub fn process_fill_order(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        args: FillOrderArgs,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let maker_info = next_account_info(account_info_iter)?;
        let taker_info = next_account_info(account_info_iter)?;
        let sysvar_instr = next_account_info(account_info_iter)?;
        let onchain_order_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;

        let get_maker_amount_infos: Vec<AccountInfo> =
            account_info_iter
                .take(args.get_maker_amount_infos_num as usize)
                .cloned()
                .collect();
        
        let get_taker_amount_infos: Vec<AccountInfo> =
            account_info_iter
                .take(args.get_taker_amount_infos_num as usize)
                .cloned()
                .collect();

        let predicate_infos: Vec<AccountInfo> = 
            account_info_iter
                .take(args.predicate_infos_num as usize)
                .cloned()
                .collect();

        let callback_infos: Vec<AccountInfo> =
            account_info_iter
                .take(args.callback_infos_num as usize)
                .cloned()
                .collect();

        let mut onchain_order = match onchain_order_info.data_is_empty() {
            true => {
                let order = args.order
                    .ok_or(ProgramError::from(SolarisAutoError::OrderIsNone))?;
                let order_hash = keccak::hash(&order.try_to_vec().unwrap());

                is_valid_signature(maker_info.key, order_hash.as_ref(), sysvar_instr)?;

                let sign_seeds_onchain_order = 
                    [
                        PREFIX.as_bytes(),
                        ONCHAIN_ORDER.as_bytes(),
                        order_hash.as_ref(),
                        &[get_bump_onchain_order(order_hash.as_ref())]
                    ];
                
                invoke_signed(
                    &create_onchain_order(
                        taker_info.key,
                        onchain_order_info.key,
                        &order,
                    )?,
                    &[
                        taker_info.clone(),
                        onchain_order_info.clone(),
                        system_program_info.clone(),
                    ],
                    &[&sign_seeds_onchain_order],
                );

                let temp_stage = match order.callback.is_empty() {
                    true => {
                        OrderStage::Filled
                    }, 
                    false => {
                        OrderStage::Create
                    }
                };
                
                OnchainOrder {
                    key: Key::OnchainOrder,
                    order_hash: order_hash.to_bytes(),
                    making_amount: order.making_amount,
                    taking_amount: order.taking_amount,
                    remaining_maker_amount: order.making_amount,
                    get_maker_amount: order.get_maker_amount,
                    get_taker_amount: order.get_taker_amount,
                    predicate: order.predicate,
                    callback: order.callback,
                    stage: temp_stage,
                }
                
            },
            false => {
                // TODO: Validate onchain_order as PDA with seeds [prefix, order, order_hash]
                // TODO: assert_is_owner(program_id, onchain_order_info)?;

                OnchainOrder::from_account_info(onchain_order_info)?
            }
        };
    
        match onchain_order.stage {
            OrderStage::Create => {
                onchain_order.stage = OrderStage::Filled;

                onchain_order.serialize(&mut *onchain_order_info.data.borrow_mut())?;
            },
            OrderStage::Filled => {
                let taker_ta_taker_asset_info = next_account_info(account_info_iter)?;
                let maker_ta_taker_asset_info = next_account_info(account_info_iter)?;

                let maker_ta_maker_asset_info = next_account_info(account_info_iter)?;
                let taker_ta_maker_asset_info = next_account_info(account_info_iter)?;

                let delegate = next_account_info(account_info_iter)?;
                let token_program = next_account_info(account_info_iter)?;
                // TODO: add validating for accounts (matching with order)

                check_predicate(&onchain_order.predicate, &predicate_infos[..])?;

                // TODO: check that args.making_amount != args.taking_amount != 0
                let (taking_amount, making_amount) = match args.making_amount {
                    0 => {
                        // I'm a taker
                        let making_amount = get_maker_amount(
                            onchain_order.making_amount,
                            onchain_order.taking_amount,
                            args.taking_amount,
                        );

                        msg!("making_amount is {}", making_amount);

                        // TODO: if making_amount > onchain_order.remaining_maker_amount

                        (args.taking_amount, making_amount)
                    },
                    _ => {
                        // I'm a maker
                        let making_amount = 
                            match args.making_amount > onchain_order.remaining_maker_amount {
                                true => onchain_order.remaining_maker_amount,
                                false => args.making_amount
                        };

                        let taking_amount = get_taker_amount(
                            onchain_order.making_amount,
                            onchain_order.taking_amount,
                            making_amount,
                        );

                        (taking_amount, making_amount)
                    }
                };
                
                // Taker => Maker
                invoke(
                    &spl_token::instruction::transfer(
                        &spl_token::id(),
                        taker_ta_taker_asset_info.key,
                        maker_ta_taker_asset_info.key,
                        taker_info.key,
                        &[taker_info.key],
                        taking_amount,
                    )?, 
                    &[
                        taker_ta_taker_asset_info.clone(),
                        maker_ta_taker_asset_info.clone(),
                        taker_info.clone(),
                        token_program.clone(),
                    ],
                )?;

                // TODO: callback

                // Maker => Taker
                invoke_signed(
                    &spl_token::instruction::transfer(
                        &spl_token::id(),   
                        maker_ta_maker_asset_info.key,
                        taker_ta_maker_asset_info.key,
                        delegate.key,
                        &[delegate.key],
                        making_amount, 
                    )?,
                    &[
                        maker_ta_maker_asset_info.clone(),
                        taker_ta_maker_asset_info.clone(),
                        delegate.clone(),
                        token_program.clone(),
                    ],
                    &[&get_seeds_delegate()],
                )?;

                onchain_order.stage = OrderStage::Closed;
                
                onchain_order.serialize(&mut *onchain_order_info.data.borrow_mut())?;
            },
            OrderStage::Closed => {
                return Err(SolarisAutoError::OrderClosed.into())
            }
        }

        Ok(())
    }

    pub fn process_proxy_deposit_reserve_liquidity_and_obligation_collateral(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        liquidity_amount: u64,
    ) -> ProgramResult {

        Ok(())
    }

    pub fn process_init_delegate(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let payer = next_account_info(account_info_iter)?;
        let delegate = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                delegate.key,
                1_000_000,
                0,
                program_id,
            ),
            &[
                payer.clone(),
                delegate.clone(),
                system_program.clone(),
            ],
            &[&get_seeds_delegate()],
        )?;

        Ok(())
    }
}
