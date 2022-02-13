use solana_program::{
    pubkey::Pubkey,
    sysvar::instructions::{load_current_index_checked, load_instruction_at_checked},
    program_error::ProgramError,
    entrypoint::ProgramResult,
    account_info::AccountInfo,
    ed25519_program,
    instruction::Instruction,
};
use byteorder::ByteOrder;

use crate::{
    error::SolarisAutoError,
};

pub fn is_valid_signature(
    maker_id: &Pubkey,
    order_hash: &[u8],
    instructions_info: &AccountInfo,
) -> ProgramResult {
    let ed_sign_ix: usize = load_current_index_checked(instructions_info)
        .and_then(assert_current_index_is_not_zero)
        .and_then(|current_ix| Ok((current_ix - 1) as usize))?;

    let _ = load_instruction_at_checked(ed_sign_ix, instructions_info)
        .and_then(assert_sign_verify_instruction)
        .and_then(|instr| assert_signer_is_maker(instr, maker_id))  
        .and_then(|instr| assert_message(instr, order_hash))?;

    Ok(())
}

fn assert_signer_is_maker(
    instruction: Instruction,
    maker_id: &Pubkey,
) -> Result<Instruction, ProgramError> {
    // According to the struct Ed25519SignatureOffsets
    let index: usize = 
        0 // count and padding
        + 2 // signature_offset
        + 2 // signature_instruction_index
        + 2; // public_key_offset
        
    let signer_offset = byteorder::LE::read_u16(&instruction.data[index..index + 2]) as usize;

    let signer: &[u8] = &instruction.data[signer_offset..signer_offset + 32];

    if signer != maker_id.as_ref() {
        Err(ProgramError::from(SolarisAutoError::InvalidSigner))
    } else {
        Ok(instruction)
    }
}

fn assert_message(
    instruction: Instruction,
    order_hash: &[u8],
) -> ProgramResult {
    // According to the struct Ed25519SignatureOffsets
    let mut index: usize = 
        0 // count and padding
        + 2 // signature_offset
        + 2 // signature_instruction_index
        + 2 // public_key_offset
        + 2 // public_key_instruction_index
        + 2; // message_data_offset

    let msg_offset = byteorder::LE::read_u16(&instruction.data[index..index + 2]) as usize;

    index += 2; // message_data_size
    let msg_size = byteorder::LE::read_u16(&instruction.data[index..index + 2]);
    
    if msg_size != 32 {
        return Err(ProgramError::from(SolarisAutoError::InvalidMsgSize))
    }

    let msg: &[u8] = &instruction.data[msg_offset..msg_offset + 32];
    
    if msg != order_hash {
        Err(ProgramError::from(SolarisAutoError::InvalidMsg))
    } else {
        Ok(())
    }
}

fn assert_current_index_is_not_zero(
    index: u16,
) -> Result<u16, ProgramError> {
    if index == 0 { 
        Err(ProgramError::from(SolarisAutoError::InvalidInstrIndex))
    } else {
        Ok(index)
    }
}

fn assert_sign_verify_instruction(
    instruction: Instruction,
) -> Result<Instruction, ProgramError> {
    if instruction.program_id != ed25519_program::id() {
        return Err(ProgramError::from(SolarisAutoError::InvalidProgramIdSignVerify))
    }
    if instruction.data.len() < 2 {
        return Err(ProgramError::from(SolarisAutoError::InvalidInstrSignVerify))
    } 
    if instruction.data[0] != 1 {
        return Err(ProgramError::from(SolarisAutoError::InvalidCountSignVerify))
    }

    Ok(instruction)
}