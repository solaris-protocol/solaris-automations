use borsh::{
    BorshSerialize,     
    BorshDeserialize,
    BorshSchema,
};

#[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
pub enum PredicateInstruction {
    /// Predicate instruction 
    ///
    /// Accounts expected:
    /// 
    IsHFBelow {
        return_val: bool,
    },
}
