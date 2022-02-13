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
