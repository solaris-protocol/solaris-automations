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
    signer::{keypair::Keypair, Signer},
    pubkey::Pubkey,
    transaction::{Transaction, TransactionError},
    instruction::{Instruction, InstructionError},  
    system_program,
};
use solaris_automations::{
    id,
    processor::Processor,
    instruction::fill_order,
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
    let maker = Keypair::new();
    let mut maker_data = Account::new(5_000, 0, &system_program::ID);

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
        data: vec![],
    };

    let mut tx_fill_order = Transaction::new_with_payer(
        &[
            fill_order(
                &id(),
                &maker.pubkey(),
                &taker,
                &predicate_contract,
                &[],

                predicate,
            )
        ],
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