use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
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
    instruction::{
        SolarisAutoInstruction,
        FillOrderArgs,
    },
    helpers::check_predicate,
    verify_sign::is_valid_signature,
    utils::{
        get_seeds_delegate,
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

        let maker = next_account_info(account_info_iter)?;
        let taker = next_account_info(account_info_iter)?;
        let delegate = next_account_info(account_info_iter)?;
        let sysvar_instr = next_account_info(account_info_iter)?;
        let spl_token_info = next_account_info(account_info_iter)?;

        let predicate_infos: Vec<AccountInfo> = 
            account_info_iter
                .take(args.predicate_infos_count as usize)
                .cloned()
                .collect();

        let mut maker_asset_data_infos: Vec<AccountInfo> = 
            account_info_iter
                .take(2)
                .cloned()
                .collect();

        let mut taker_asset_data_infos: Vec<AccountInfo> =
            account_info_iter  
                .take(3)
                .cloned()
                .collect();

        maker_asset_data_infos.push(delegate.clone());
        //taker_asset_data_infos.push(spl_token_info.clone());

        let order = args.order;
        let order_hash = keccak::hash(&order.try_to_vec().unwrap()); 

        // TODO: Create a PDA for order
        /* 
        is remaining_maker_amount = ORDER_DOES_NOT_EXIST {
            // First fill. Validate order and create a PDA for order
            // is_vaid_signature(maker.key, order_hash.as_ref(), sysvar_instr)?;
        }
        */
        is_valid_signature(maker.key, order_hash.as_ref(), sysvar_instr)?;

        check_predicate(&order.predicate, &predicate_infos[..])?;
    
        // TODO: add validating for accounts (matching with order)
        // Taker => Maker
        invoke(
            &spl_token::instruction::transfer(
                &spl_token::id(),
                taker_asset_data_infos[0].key,
                taker_asset_data_infos[1].key,
                taker_asset_data_infos[2].key,
                &[taker_asset_data_infos[2].key],
                order.taking_amount,
            )?, 
            &taker_asset_data_infos,
        )?;

        // TODO: callback

        // Maker => Taker
        invoke_signed(
            &spl_token::instruction::transfer(
                &spl_token::id(),
                maker_asset_data_infos[0].key,
                maker_asset_data_infos[1].key,
                delegate.key,
                &[delegate.key],
                order.making_amount,
            )?,
            &maker_asset_data_infos,
            &[&get_seeds_delegate()],
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
