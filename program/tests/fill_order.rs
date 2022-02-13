#![cfg(feature = "test-bpf")]
use std::{
    str::FromStr,
    option::Option,
};
use solana_program_test::*;
use solana_program::{
    instruction::AccountMeta,
};
use solana_sdk::{
    account::Account,
    signature::Signer,
    signer::keypair::Keypair as SdkKeypair,
    pubkey::Pubkey,
    transaction::{Transaction, TransactionError},
    instruction::{Instruction, InstructionError},  
    system_program,
    ed25519_instruction,
    keccak,
};
use ed25519_dalek::Keypair;
use solaris_automations::{
    id,
    processor::Processor,
    instruction::fill_order,
    helpers::{HELPER_AND_ID},
};

pub fn add_accounts_to_program_test(
    params: &[(&Pubkey, Account)]
) -> ProgramTest {
    let mut program_test = ProgramTest::new(
        "solaris_automations",
        id(),
        processor!(Processor::process),
    );  

    for param in params {
        program_test.add_account(
            *param.0,
            param.1.clone(),
        )
    }

    program_test
}

#[tokio::test]
async fn test_fill_order() {
    let maker = SdkKeypair::new();
    let mut maker_data = Account::new(10_000, 0, &system_program::ID);

    let maker_dalek = ed25519_dalek::Keypair::from_bytes(
        &maker.to_bytes()).unwrap();

    let taker = Pubkey::new_rand();
    let mut taker_data = Account::new(0, 0, &system_program::ID);

    let predicate_contract = Pubkey::from_str("2TPBhV6fb7V2yJAg9qpfQHkpbEFNRLT7cTRpzPi2vzyY").unwrap();

    let mut program_test = add_accounts_to_program_test(
        &[
            (&maker.pubkey(), maker_data),
            (&taker, taker_data),
        ]
    );

    program_test.add_program(
        "predicate_example",
        predicate_contract,
        None,
    );

    let mut context = program_test.start_with_context().await;

    let predicate = Instruction{
        program_id: predicate_contract,
        accounts: vec![],
        data: vec![0,1],
    };

    let fill_order = fill_order(
        &id(),
        &maker.pubkey(),
        &taker,
        &predicate_contract,
        &[],

        &predicate,
    );

    let encoded_predicate = bincode::serialize(&predicate).unwrap();

    let sign = ed25519_instruction::new_ed25519_instruction(
        &maker_dalek, keccak::hash(&encoded_predicate).as_ref(),
    );

    let mut tx_fill_order = Transaction::new_with_payer(
        &[sign, fill_order],
        Some(&maker.pubkey()),
    );
    tx_fill_order.sign(&[&maker], context.last_blockhash);

    assert!(
        context
            .banks_client
            .process_transaction(tx_fill_order)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn test_fill_order_and() {
    let maker = SdkKeypair::new();
    let mut maker_data = Account::new(100_000_000, 0, &system_program::ID);

    let maker_dalek = ed25519_dalek::Keypair::from_bytes(
        &maker.to_bytes()).unwrap();

    let taker = Pubkey::new_rand();
    let mut taker_data = Account::new(0, 0, &system_program::ID);

    let predicate_contract = Pubkey::from_str("2TPBhV6fb7V2yJAg9qpfQHkpbEFNRLT7cTRpzPi2vzyY").unwrap();

    let mut program_test = add_accounts_to_program_test(
        &[
            (&maker.pubkey(), maker_data),
            (&taker, taker_data),
        ]
    );

    program_test.add_program(
        "predicate_example",
        predicate_contract,
        None,
    );

    let mut context = program_test.start_with_context().await;

    let and_instructions_ok = bincode::serialize(&vec![
        Instruction{
            program_id: predicate_contract,
            accounts: vec![],
            data: vec![0,1],
        },
        Instruction{
            program_id: predicate_contract,
            accounts: vec![],
            data: vec![0,1],
        },
    ]).unwrap();

    let predicate_ok = Instruction{ 
        program_id: Pubkey::new(HELPER_AND_ID),
        accounts: vec![],
        data: and_instructions_ok,
    };
    
    let fill_order_ok = fill_order(
        &id(),
        &maker.pubkey(),
        &taker,
        &predicate_contract,
        &[],
        
        &predicate_ok,
    );
    
    let encoded_predicate_ok = bincode::serialize(&predicate_ok).unwrap();

    let sign = ed25519_instruction::new_ed25519_instruction(
        &maker_dalek, keccak::hash(&encoded_predicate_ok).as_ref(),
    );
    
    let mut tx_fill_order_ok = Transaction::new_with_payer(
        &[sign, fill_order_ok],
        Some(&maker.pubkey()),
    );
    tx_fill_order_ok.sign(&[&maker], context.last_blockhash);

    assert!(
        context
            .banks_client
            .process_transaction(tx_fill_order_ok)
            .await
            .is_ok()
    );
   
    // Test for predicate Err
    let and_instructions_err = bincode::serialize(&vec![
        Instruction{
            program_id: predicate_contract,
            accounts: vec![],
            data: vec![0,1],
        },
        // Instruction that return Err
        Instruction{
            program_id: predicate_contract,
            accounts: vec![],
            data: vec![0,0], 
        },
    ]).unwrap();

    let predicate_err = Instruction{
        program_id: Pubkey::new(HELPER_AND_ID),
        accounts: vec![],
        data: and_instructions_err,
    };

    
    let fill_order_err = fill_order(
        &id(),
        &maker.pubkey(),
        &taker,
        &predicate_contract,
        &[],
        
        &predicate_err,
    );
    
    let encoded_predicate_err = bincode::serialize(&predicate_err).unwrap();
    
    let sign_err = ed25519_instruction::new_ed25519_instruction(
        &maker_dalek, keccak::hash(&encoded_predicate_err).as_ref(),
    );

    let mut tx_fill_order_err = Transaction::new_with_payer(
        &[sign_err, fill_order_err],
        Some(&maker.pubkey()),
    );
    tx_fill_order_err.sign(&[&maker], context.last_blockhash);

    assert!(    
        context
            .banks_client
            .process_transaction(tx_fill_order_err)
            .await
            .is_err()
    );
}