use borsh::{
    BorshSerialize,     
    BorshDeserialize,
    BorshSchema,
};

#[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
pub enum CustomGetAmountsInstruction {
    /// Custom get [taker/maker] amount instructions 
    ///
    /// Accounts expected:
    /// 
    GetMakerAmount,
    ///
    /// Accounts expected:
    /// 
    GetTakerAmount,
}
