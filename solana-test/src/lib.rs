use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::next_account_info;
use solana_program::example_mocks::solana_sdk::system_instruction::{self, create_account};
use solana_program::program::invoke;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{entrypoint, lamports};
use solana_program::{
    entrypoint::{__AccountInfo, ProgramResult},
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process);

pub enum Instrctions {
    IntializeEscrow { amount: u64, hash: String },
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
    let instruction = Instrctions::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        Instrctions::IntializeEscrow { amount, hash } => {
            let accounts_into_iter = &mut accounts.iter();
            let initializer = next_account_info(accounts_into_iter)?;
            let pda_account = next_account_info(accounts_into_iter)?;
            let system_program = next_account_info(accounts_into_iter)?;

            let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

            let space = 8; // Set the appropriate space for your account
            let lamports = Rent::get()?.minimum_balance(space);
            let create_pda_account_ix =
                create_account(initializer.key, &pda, lamports, space as u64, owner);

            invoke(
                &create_pda_account_ix,
                &[
                    initializer.clone(),
                    pda_account.clone(),
                    system_program.clone(),
                ],
            )?;
        }
    }
    Ok(())
}
