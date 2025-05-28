use account::AccountInfo;
use program_error::ProgramError;

pub mod account;
pub mod atomic_u64;
pub mod clock;
pub mod debug_account_data;
pub mod entrypoint;
pub mod helper;
pub mod input_to_sign;
pub mod instruction;
pub mod log;
pub mod message;
pub mod program;
pub mod program_error;
pub mod program_stubs;
pub mod pubkey;
pub mod sanitized;
pub mod stable_layout;
pub mod syscalls;
pub mod system_instruction;
pub mod transaction_to_sign;
pub mod utxo;

pub const MAX_BTC_TX_SIZE: usize = 1024;

// Helper Funtions
pub fn next_account_info<'a, 'b, I: Iterator<Item = &'a AccountInfo<'b>>>(
    iter: &mut I,
) -> Result<I::Item, ProgramError> {
    iter.next().ok_or(ProgramError::NotEnoughAccountKeys)
}
