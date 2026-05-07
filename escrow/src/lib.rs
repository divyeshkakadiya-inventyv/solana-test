use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::next_account_info,
    entrypoint,
    entrypoint::{__AccountInfo, ProgramResult},
    keccak,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use solana_system_interface::instruction::{create_account, transfer};

entrypoint!(process_instruction);

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct EscrowState {
    pub intializer: Pubkey,
    pub amount: u64,
    pub hash: [u8; 32],
    pub claimed: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum Instructions {
    IntializeEscrow { amount: u64, password: [u8; 32] },
    ClaimEscrow { password: [u8; 32] },
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[__AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = Instructions::try_from_slice(instruction_data)?;

    match instruction {
        Instructions::IntializeEscrow {
            amount,
            password: hash,
        } => {
            let account_iter = &mut accounts.iter();

            let intializer = next_account_info(account_iter)?;
            let escrow_pda_account = next_account_info(account_iter)?;
            let system_program = next_account_info(account_iter)?;

            if !intializer.is_signer {
                return Err(solana_program::program_error::ProgramError::MissingRequiredSignature);
            }

            let (pda, bump) =
                Pubkey::find_program_address(&[b"escrow", intializer.key.as_ref()], program_id);

            if pda != *escrow_pda_account.key {
                return Err(solana_program::program_error::ProgramError::InvalidSeeds);
            }

            let space = 200;
            let rent = Rent::get()?;
            let lamports = rent.minimum_balance(space);

            let create_account_ix = create_account(
                intializer.key,
                escrow_pda_account.key,
                lamports,
                space as u64,
                program_id,
            );

            let signer_seeds: &[&[u8]] = &[b"escrow", intializer.key.as_ref(), &[bump]];

            invoke_signed(
                &create_account_ix,
                &[
                    intializer.clone(),
                    escrow_pda_account.clone(),
                    system_program.clone(),
                ],
                &[signer_seeds],
            )?;

            let transfer_ix = transfer(intializer.key, &pda, amount);

            invoke(
                &transfer_ix,
                &[
                    intializer.clone(),
                    escrow_pda_account.clone(),
                    system_program.clone(),
                ],
            )?;

            let escrow_data = EscrowState {
                intializer: *intializer.key,
                amount,
                hash, // ← directly store, no keccak
                claimed: false,
            };

            escrow_data.serialize(&mut &mut escrow_pda_account.data.borrow_mut()[..])?;
        }
        Instructions::ClaimEscrow { password: hash } => {
            let account_iter = &mut accounts.iter();

            let claimer = next_account_info(account_iter)?;
            let escrow_pda_account = next_account_info(account_iter)?;

            if !claimer.is_signer {
                return Err(solana_program::program_error::ProgramError::MissingRequiredSignature);
            }

            let mut escrow_data = EscrowState::try_from_slice(&escrow_pda_account.data.borrow())?;

            if escrow_data.hash != hash {
                return Err(solana_program::program_error::ProgramError::InvalidInstructionData);
            }

            if escrow_data.claimed {
                return Err(solana_program::program_error::ProgramError::InvalidAccountData);
            }

            **escrow_pda_account.try_borrow_mut_lamports()? -= escrow_data.amount;
            **claimer.try_borrow_mut_lamports()? += escrow_data.amount;
            escrow_data.claimed = true;
            escrow_data.serialize(&mut &mut escrow_pda_account.data.borrow_mut()[..])?;
        }
    }

    Ok(())
}
