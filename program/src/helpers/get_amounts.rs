use solana_program::{
    account_info::AccountInfo,
    instruction::Instruction,
    pubkey::Pubkey,
};
use byteorder::ByteOrder;
//Pubkey is "Go5vxb6EqNzoupdUuyaXkUX2SYr4eQmoVfa2sFm5PX6Z"
pub const HELPER_GET_AMOUNTS_ID: &[u8] = &[234, 173, 152, 136, 255, 42, 231, 127, 101, 7, 52, 33, 126, 91, 30, 78, 154, 206, 135, 149, 215, 253, 222, 19, 175, 11, 224, 12, 185, 149, 14, 42];

/// TODO: 
/// 
/// Accounts required:
/// 
/// 
/// 
/// Instruction data format is 
/// ```
/// pub struct HelperGetAmount {
///     get_maker_amount: bool, 
///     order_maker_amount: u64,
///     order_taker_amount: u64,
///     swap_amount: u64, // swap_taker_amount if `get_maker_amount` == _true_
///                       // swap_maker_amount if `get_maker_amount` == _false_
/// }
/// ```
pub fn process_get_amounts(
    instr: &Instruction,
    _accounts: &[AccountInfo],
) -> u64 {
    let get_maker_amount = instr.data[0] != 0;
    let order_maker_amount = byteorder::LE::read_u64(&instr.data[1..9]); 
    let order_taker_amount = byteorder::LE::read_u64(&instr.data[10..19]); 
    let swap_amount = byteorder::LE::read_u64(&instr.data[19..28]); 

    match get_maker_amount {
        true => {
            swap_amount * order_maker_amount / order_taker_amount
        },
        false => {
            swap_amount * order_maker_amount / order_taker_amount
        }
    }
}

// This is a CRUTCH
pub fn get_maker_amount(
    order_maker_amount: u64,
    order_taker_amount: u64,
    swap_taker_amount: u64,
) -> u64 {
    swap_taker_amount * order_maker_amount / order_taker_amount
}

// This is also a CRUTCH
pub fn get_taker_amount(
    order_maker_amount: u64,
    order_taker_amount: u64,
    swap_maker_amount: u64,
) -> u64 {
    swap_maker_amount * order_maker_amount / order_taker_amount
}
