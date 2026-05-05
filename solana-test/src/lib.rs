use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::next_account_info;
use solana_program::entrypoint;
use solana_program::{
    entrypoint::{__AccountInfo, ProgramResult},
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process);

pub enum CustomInstruction {
    Intialize,
    Increment,
    Decrement,
    Reset
}

impl CustomInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        match input[0] {
            0 => Ok(CustomInstruction::Intialize),
            1 => Ok(CustomInstruction::Increment),
            2 => Ok(CustomInstruction::Decrement),
            3 => Ok(CustomInstruction::Reset),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Counter {
    pub authority: Pubkey,
    pub value: u64,
}

pub fn process(
    process_id: &Pubkey,
    accounts: &[__AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    process_intruction(process_id, accounts, instruction_data)
}

pub fn process_intruction(
    process_id: &Pubkey,
    accounts: &[__AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = CustomInstruction::unpack(instruction_data)?;

    let accounts_iter = &mut accounts.iter();

    let counter_account = next_account_info(accounts_iter)?;
    let authority_account = next_account_info(accounts_iter)?;

    if counter_account.owner != process_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut data = counter_account.try_borrow_mut_data()?;

    
    match instruction {
        CustomInstruction::Intialize => {
            if data.len() > 0 && data[0] != 0 {
                return Err(ProgramError::AccountAlreadyInitialized);
            }
            
            let counter = Counter{
                authority : *authority_account.key,
                value : 0,
            };
            
            counter.serialize(&mut *data)?;
        }
        CustomInstruction::Increment | CustomInstruction::Decrement => {
            let mut counter = Counter::try_from_slice(&data)?;
            
            if !authority_account.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            if counter.authority != *authority_account.key {
                return Err(ProgramError::IllegalOwner);
            }

            match instruction {
                CustomInstruction::Increment => counter.value += 1,
                CustomInstruction::Decrement => counter.value -= 1,
                _ => {},
            }

            counter.serialize(&mut *data)?;
        }
        CustomInstruction::Reset => {
            let mut counter = Counter::try_from_slice(&data)?;

            if !authority_account.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            if counter.authority != *authority_account.key {
                return Err(ProgramError::IllegalOwner);
            }
            counter.value = 0;
            counter.serialize(&mut *data)?;
        }
    }


    Ok(())
}
