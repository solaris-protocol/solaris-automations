use solana_program::{
    pubkey::Pubkey,
    instruction::{Instruction, AccountMeta},
};
use borsh::{
    BorshSerialize,     
    BorshDeserialize,
    BorshSchema,
};

#[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
pub enum SolarisAutoInstruction {
    /// 0
    /// Fill order
    /// 
    /// Accounts expected:
    /// 
    /// 0. `[writable]` Maker account
    /// 1. `[writable]` Taker account 
    /// 2. `[]` Predicate contract account
    /// *TODO* 3. `[writable]` PDA order. Seeds: []  
    /// 4.. Accounts that required by predicate instruction
    FillOrder { // TODO: Data has to be in PDA order
        predicate: Vec<u8>,
    }
}

pub fn fill_order(
    program_id: &Pubkey,
    maker: &Pubkey,
    taker: &Pubkey,
    predicate_contract: &Pubkey,
    predicate_accounts: &[&Pubkey],

    predicate: Instruction,
) -> Instruction {
    let data = SolarisAutoInstruction::FillOrder {
        predicate: bincode::serialize(&predicate).unwrap(),
    }
    .try_to_vec().unwrap();

    let mut accounts = vec![
        AccountMeta::new(*maker, true), // TODO: change to option signer 
                                        //       for maker or taker
        AccountMeta::new(*taker, false),
        AccountMeta::new(*predicate_contract, false),
    ];

    //TODO: add predicate_accounts to vec `accounts`

    Instruction{
        program_id: *program_id,
        accounts,
        data,
    }
}
