use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError, 
    program::{invoke, invoke_signed}, 
    pubkey::Pubkey,
    system_instruction,
    msg,
    keccak,
    sysvar::Sysvar,
    rent::Rent,
    instruction::{Instruction, AccountMeta},
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use byteorder::{ByteOrder, LittleEndian};

use crate::{
    helpers::{
        predicate_helpers::check_predicate,
        //get_amounts::process_get_amounts,
        get_amounts::{get_maker_amount, get_taker_amount}, 
    },
    callbacks::{
        process_callback::process_callback,
    },
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
        get_seeds_collateral_ta,
        get_bump_onchain_order,
        create_onchain_order,
        create_collateral_token_account,
        solend_init_obligation,
    },
    verify_sign::is_valid_signature,
    error::SolarisAutoError,
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
            SolarisAutoInstruction::SolendProxyDepositReserveLiquidityAndObligationCollateral {
                liquidity_amount,
            } => {
                msg!("Instruction: SolendProxyDepositReserveLiquidityAndObligationCollateral");
                Self::process_solend_proxy_deposit_reserve_liquidity_and_obligation_collateral(program_id, accounts, liquidity_amount)
            },
            SolarisAutoInstruction::SolendProxyBorrowObligationLiquidity {
                liquidity_amount
            } => {
                msg!("Instruction: SolendProxyBorrowObligationLiquidity");
                Self::process_solend_proxy_borrow_obligation_liquidity(program_id, accounts, liquidity_amount)
            }
            SolarisAutoInstruction::SolendInitAccountsForDelegate
            => {
                msg!("Instruction: SolendInitAccountsForDelegate");
                Self::process_solend_init_accounts_for_delegate(program_id, accounts)
            }
            SolarisAutoInstruction::InitDelegate
            => {
                msg!("Instruction: InitDelegate");
                Self::process_init_delegate(program_id, accounts)
            }
        }
    }

    pub fn process_fill_order(
        _program_id: &Pubkey,
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

                let taker_ta_taker_asset_info = next_account_info(account_info_iter)?;
                let maker_ta_taker_asset_info = next_account_info(account_info_iter)?;

                let maker_ta_maker_asset_info = next_account_info(account_info_iter)?;
                let taker_ta_maker_asset_info = next_account_info(account_info_iter)?;

                let delegate = next_account_info(account_info_iter)?;
                let token_program = next_account_info(account_info_iter)?;
                // TODO: add validating for accounts (matching with order)

                if !onchain_order.predicate.is_empty(){
                    check_predicate(&onchain_order.predicate, &predicate_infos[..])?;
                }   

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

                // Assert accounts match with onchain_order.accounts
                process_callback(&onchain_order.callback, &callback_infos)?;

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
                
                //onchain_order.serialize(&mut *onchain_order_info.data.borrow_mut())?;
            },
            OrderStage::Closed => {
                return Err(SolarisAutoError::OrderClosed.into())
            }
        }

        Ok(())
    }

    pub fn process_solend_proxy_deposit_reserve_liquidity_and_obligation_collateral(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        liquidity_amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let mut solend_deposit_infos: Vec<AccountInfo> = 
            account_info_iter
                .take(14)
                .cloned()
                .collect();  

        let token_program = next_account_info(account_info_iter)?;
        let source_user_liquidity_ta = next_account_info(account_info_iter)?;
        let maker = next_account_info(account_info_iter)?;
        let solend_program = next_account_info(account_info_iter)?;
        
        solend_deposit_infos.push(token_program.clone()); // 14

        invoke(
            &spl_token::instruction::transfer(
                token_program.key,
                source_user_liquidity_ta.key,
                solend_deposit_infos[0].key,
                maker.key,
                &[maker.key],
                liquidity_amount,
            )?,
            &[
                source_user_liquidity_ta.clone(),
                solend_deposit_infos[0].clone(),
                maker.clone(),
                token_program.clone(),
            ],
        )?;

        let mut data = [0; 9];
        LittleEndian::write_u64(&mut data[1..9], liquidity_amount);
        data[0] = 14;
 
        let solend_deposit = Instruction {
            program_id: *solend_program.key,
            accounts: vec![
                AccountMeta::new(*solend_deposit_infos[0].key, false),
                AccountMeta::new(*solend_deposit_infos[1].key, false),
                AccountMeta::new(*solend_deposit_infos[2].key, false),
                AccountMeta::new(*solend_deposit_infos[3].key, false),
                AccountMeta::new(*solend_deposit_infos[4].key, false),
                AccountMeta::new(*solend_deposit_infos[5].key, false),
                AccountMeta::new_readonly(*solend_deposit_infos[6].key, false),
                AccountMeta::new(*solend_deposit_infos[7].key, false),
                AccountMeta::new(*solend_deposit_infos[8].key, false),
                AccountMeta::new(*solend_deposit_infos[9].key, true),
                AccountMeta::new_readonly(*solend_deposit_infos[10].key, false),
                AccountMeta::new_readonly(*solend_deposit_infos[11].key, false),
                AccountMeta::new(*solend_deposit_infos[12].key, true),
                AccountMeta::new_readonly(*solend_deposit_infos[13].key, false),
                AccountMeta::new_readonly(*solend_deposit_infos[14].key, false),
            ],
            data: data.to_vec(),
        };        

        invoke_signed(
            &solend_deposit,
            &solend_deposit_infos,
            &[&get_seeds_delegate()],
        )?;

        Ok(())
    }

    pub fn process_solend_proxy_borrow_obligation_liquidity(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        liquidity_amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let mut solend_refresh_reserve_collateral_infos: Vec<AccountInfo> =
            account_info_iter
                .take(3)
                .cloned()
                .collect();

        let mut solend_refresh_reserve_liquidity_infos: Vec<AccountInfo> = 
            account_info_iter
                .take(3)
                .cloned()
                .collect();

        let mut solend_borrow_infos: Vec<AccountInfo> = 
            account_info_iter
                .take(7)
                .cloned()
                .collect();  

        let obligation_owner_info = next_account_info(account_info_iter)?;
        let clock = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let solend_program = next_account_info(account_info_iter)?;

        solend_refresh_reserve_collateral_infos.push(clock.clone()); // 3
        solend_refresh_reserve_liquidity_infos.push(clock.clone()); // 3
        
        solend_borrow_infos.push(obligation_owner_info.clone()); // 7
        solend_borrow_infos.push(clock.clone()); // 8
        solend_borrow_infos.push(token_program.clone()); // 9

        let refresh_reserve_collateral = Instruction {
            program_id: *solend_program.key,
            accounts: vec![
                AccountMeta::new(*solend_refresh_reserve_collateral_infos[0].key, false),
                AccountMeta::new_readonly(*solend_refresh_reserve_collateral_infos[1].key, false),
                AccountMeta::new_readonly(*solend_refresh_reserve_collateral_infos[2].key, false),
                AccountMeta::new_readonly(*clock.key, false),
            ],
            data: vec![3],  
        };

        let refresh_reserve_liquidity = Instruction {
            program_id: *solend_program.key,
            accounts: vec![
                AccountMeta::new(*solend_refresh_reserve_liquidity_infos[0].key, false),
                AccountMeta::new_readonly(*solend_refresh_reserve_liquidity_infos[1].key, false),
                AccountMeta::new_readonly(*solend_refresh_reserve_liquidity_infos[2].key, false),
                AccountMeta::new_readonly(*clock.key, false),
            ],
            data: vec![3],  
        };

        // TODO: dont' know. It works. All questions for the token-lending program
        let mut solend_refresh_obligation_infos = vec![
            solend_borrow_infos[4].clone(),
            clock.clone(),
            solend_refresh_reserve_liquidity_infos[0].clone(),
            solend_refresh_reserve_liquidity_infos[0].clone(),
        ];

        let refresh_obligation = Instruction {
            program_id: *solend_program.key,
            accounts: vec![
                AccountMeta::new(*solend_refresh_obligation_infos[0].key, false),
                AccountMeta::new_readonly(*solend_refresh_obligation_infos[1].key, false),
                AccountMeta::new_readonly(*solend_refresh_obligation_infos[2].key, false),
                AccountMeta::new_readonly(*solend_refresh_obligation_infos[3].key, false),
            ],
            data: vec![7],
        };

        let mut data = [0; 9];
        LittleEndian::write_u64(&mut data[1..9], liquidity_amount);
        data[0] = 10;

        let solend_borrow = Instruction {
            program_id: *solend_program.key,
            accounts: vec![
                AccountMeta::new(*solend_borrow_infos[0].key, false),
                AccountMeta::new(*solend_borrow_infos[1].key, false),
                AccountMeta::new(*solend_borrow_infos[2].key, false),
                AccountMeta::new(*solend_borrow_infos[3].key, false),
                AccountMeta::new(*solend_borrow_infos[4].key, false),
                AccountMeta::new_readonly(*solend_borrow_infos[5].key, false),
                AccountMeta::new_readonly(*solend_borrow_infos[6].key, false),
                AccountMeta::new(*solend_borrow_infos[7].key, true),
                AccountMeta::new_readonly(*solend_borrow_infos[8].key, false),
                AccountMeta::new_readonly(*solend_borrow_infos[9].key, false),
            ],
            data: data.to_vec(),
        };

        invoke(
            &refresh_reserve_collateral,
            &solend_refresh_reserve_collateral_infos,
        )?;

        invoke(
            &refresh_reserve_liquidity,
            &solend_refresh_reserve_liquidity_infos,
        )?;

        invoke(
            &refresh_obligation,
            &solend_refresh_obligation_infos,
        )?;

        invoke_signed(
            &solend_borrow,
            &solend_borrow_infos,
            &[&get_seeds_delegate()],
        )?;

        Ok(())
    }

    pub fn process_solend_init_accounts_for_delegate(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let owner_info = next_account_info(account_info_iter)?;
        let delegate_info = next_account_info(account_info_iter)?;
        let solend_program_info = next_account_info(account_info_iter)?;
        let obligation_account_info = next_account_info(account_info_iter)?;
        let lending_market_info = next_account_info(account_info_iter)?;
        let collateral_token_account_info = next_account_info(account_info_iter)?;
        let collateral_mint_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let token_program_info = next_account_info(account_info_iter)?;
        let clock_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;

        let rent = Rent::get()?;
        let size: u64 = 1300;

        let min_rent_exempt = rent.minimum_balance(size as usize);

        invoke_signed(
            &system_instruction::create_account_with_seed(
                owner_info.key,
                obligation_account_info.key,
                delegate_info.key,
                &"GvjoVKNjBvQcFaSKUW1gTE7DxhSpjHbE69umVR5nPuQp"[0..32],
                //&"4UpD2fh7xH3VP9QQaXtsS1YY3bxzWhtfpks7FatyKvdY"[0..32],
                min_rent_exempt,
                size,
                solend_program_info.key,
            ),
            &[
                owner_info.clone(),
                obligation_account_info.clone(),
                delegate_info.clone(),
            ],
            &[&get_seeds_delegate()],
        )?;

        invoke_signed(
            &solend_init_obligation(
                solend_program_info.key,
                obligation_account_info.key,
                lending_market_info.key,
                delegate_info.key,
            ),
            &[
                obligation_account_info.clone(),
                lending_market_info.clone(),
                delegate_info.clone(),
                clock_info.clone(),
                rent_info.clone(),
                token_program_info.clone(),
            ],
            &[&get_seeds_delegate()],
        )?;

        invoke_signed(
            &create_collateral_token_account(
                owner_info.key,
                collateral_token_account_info.key,
            )?,
            &[
                owner_info.clone(),
                collateral_token_account_info.clone(),
                system_program_info.clone(),
            ],
            &[&get_seeds_collateral_ta()],
        )?;

        invoke_signed(
            &spl_token::instruction::initialize_account3(
                &spl_token::ID,
                collateral_token_account_info.key,
                collateral_mint_info.key,
                delegate_info.key,
            )?,
            &[
                collateral_token_account_info.clone(),
                collateral_mint_info.clone(),
                token_program_info.clone(),
            ],
            &[&get_seeds_collateral_ta()],
        )?;

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
