use arch_program::{
    account::AccountInfo, entrypoint, msg, program_error::ProgramError, pubkey::Pubkey,
};
use swap_inscription_rune::{swap_inscription_rune};
use swap_rune_inscription::{swap_rune_inscription};
pub mod swap_inscription_rune;
pub mod swap_rune_inscription;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    match instruction_data[0] {
        0 => {
            // Swap Inscription And Rune

            msg!("Swap Inscription And Rune ");

            swap_inscription_rune(accounts, program_id, instruction_data)
        }
        1 => {
            // Swap Rune And Inscription

            msg!("Swap Rune And Inscription ");

            swap_rune_inscription(accounts, program_id, instruction_data)
        }
        _ => {
            msg!("Invalid argument provided !");
            return Err(ProgramError::InvalidArgument);
        }
    }
}
