use solana_program::{
    pubkey::Pubkey,
    instruction::{Instruction, AccountMeta},
    sysvar,
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
    /// 2. `[]` Sysvar instructions
    /// 3. `[]` Predicate contract account
    /// *TODO* 4. `[writable]` PDA order. Seeds: []  
    /// 5.. Accounts that required by predicate instruction
    FillOrder { 
        predicate: Vec<u8>,
    }
}

pub fn fill_order(
    program_id: &Pubkey,
    maker: &Pubkey,
    taker: &Pubkey,
    predicate_contract: &Pubkey,
    predicate_accounts: &[&Pubkey],

    predicate: &Instruction,
) -> Instruction {
    let data = SolarisAutoInstruction::FillOrder {
        predicate: bincode::serialize(predicate).unwrap(),
    }
    .try_to_vec().unwrap();

    let accounts = vec![
        AccountMeta::new(*maker, true), // TODO: change to option signer 
                                        //       for maker or taker
        AccountMeta::new(*taker, false),
        AccountMeta::new(sysvar::instructions::id(), false),    
        AccountMeta::new_readonly(*predicate_contract, false),
    ];

    //TODO: add predicate_accounts to vec `accounts`

    Instruction{
        program_id: *program_id,
        accounts,
        data,
    }
}
