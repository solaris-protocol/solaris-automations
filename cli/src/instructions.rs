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
