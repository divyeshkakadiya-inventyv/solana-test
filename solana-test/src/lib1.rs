use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::next_account_info;
use solana_program::example_mocks::solana_sdk::system_instruction::{self, create_account};
use solana_program::program::invoke;
use solana_program::{entrypoint, lamports};
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
    Reset,
    TransferAuthority,
}

pub enum TransferInstruction {
    TransferSol { amount: u64 },
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct TransferData {
    amount: u64,
}

impl CustomInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        match input[0] {
            0 => Ok(CustomInstruction::Intialize),
            1 => Ok(CustomInstruction::Increment),
            2 => Ok(CustomInstruction::Decrement),
            3 => Ok(CustomInstruction::Reset),
            4 => Ok(CustomInstruction::TransferAuthority),
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

            let counter = Counter {
                authority: *authority_account.key,
                value: 0,
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
                _ => {}
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
        CustomInstruction::TransferAuthority => {
            let mut counter = Counter::try_from_slice(&data)?;

            if !authority_account.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            if counter.authority != *authority_account.key {
                return Err(ProgramError::IllegalOwner);
            }

            let new_authority_account = next_account_info(accounts_iter)?;

            if !new_authority_account.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            counter.authority = *new_authority_account.key;
            counter.serialize(&mut *data)?;
        }
    }

    Ok(())
}

pub fn create_account_cpi(
    program_id: &Pubkey,
    accounts: &[__AccountInfo],
    space: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer = next_account_info(account_info_iter)?;
    let new_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    let owner = program_id;

    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space as usize);

    let ix = create_account(payer.key, new_account.key, lamports, space, owner);

    invoke(
        &ix,
        &[payer.clone(), new_account.clone(), system_program.clone()],
    )
}

pub fn process_transfer(
    _program_id: &Pubkey,
    accounts: &[__AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let account_into_iter = &mut accounts.iter();

    let sender = next_account_info(account_into_iter)?;
    let recipient = next_account_info(account_into_iter)?;
    let system_program = next_account_info(account_into_iter)?;

    // let amount = u64::from_le_bytes(
    //     instruction_data
    //         .get(..8)
    //         .ok_or(ProgramError::InvalidInstructionData)?
    //         .try_into()
    //         .map_err(|_| ProgramError::InvalidInstructionData)?,
    // ); // instruction data = [10 , 0 , 0 , 0 , 0 , 0 , 0 , 0] => amount = 10 //manual without borsh

    let data = TransferData::try_from_slice(instruction_data)?;
    let amount = data.amount;

    if sender.is_signer == false {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let ix = system_instruction::transfer(sender.key, recipient.key, amount);

    invoke(
        &ix,
        &[sender.clone(), recipient.clone(), system_program.clone()],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use solana_program::rent::Rent;

    use super::*;

    #[test]
    pub fn test_rent() {
        let rent = Rent::default();
        let lamports = rent.minimum_balance(100);

        println!("Minimum balance for 100 bytes: {}", lamports);
    }
}
