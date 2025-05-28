use bitcoin::Transaction;

use crate::instruction::Instruction;
use crate::program_error::ProgramError;
#[cfg(target_os = "solana")]
use crate::stable_layout::stable_ins::StableInstruction;
use crate::MAX_BTC_TX_SIZE;

use crate::clock::Clock;
use crate::transaction_to_sign::TransactionToSign;
use crate::utxo::UtxoMeta;
use crate::{account::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn invoke(instruction: &Instruction, account_infos: &[AccountInfo]) -> ProgramResult {
    #[cfg(target_os = "solana")]
    {
        for account_meta in instruction.accounts.iter() {
            for account_info in account_infos.iter() {
                if account_meta.pubkey == *account_info.key {
                    if account_meta.is_writable {
                        let _ = account_info.try_borrow_mut_data()?;
                    } else {
                        let _ = account_info.try_borrow_data()?;
                    }
                    break;
                }
            }
        }

        let instruction = StableInstruction::from(instruction.clone());
        let result = unsafe {
            crate::syscalls::sol_invoke_signed_rust(
                &instruction as *const _ as *const u8,
                account_infos as *const _ as *const u8,
                account_infos.len() as u64,
            )
        };
        match result {
            crate::entrypoint::SUCCESS => Ok(()),
            _ => Err(result.into()),
        }
    }

    #[cfg(not(target_os = "solana"))]
    crate::program_stubs::sol_invoke_signed_rust(instruction, account_infos)
}

pub fn next_account_info<'a, 'b, I: Iterator<Item = &'a AccountInfo<'b>>>(
    iter: &mut I,
) -> Result<I::Item, ProgramError> {
    iter.next().ok_or(ProgramError::NotEnoughAccountKeys)
}

pub const MAX_TRANSACTION_TO_SIGN: usize = 4 * 1024;

pub fn set_transaction_to_sign(
    accounts: &[AccountInfo],
    transaction_to_sign: TransactionToSign,
) -> ProgramResult {
    let serialized_transaction_to_sign = &transaction_to_sign.serialise();
    #[cfg(target_os = "solana")]
    let result = unsafe {
        crate::syscalls::arch_set_transaction_to_sign(
            serialized_transaction_to_sign.as_ptr(),
            serialized_transaction_to_sign.len() as u64,
        )
    };
    #[cfg(not(target_os = "solana"))]
    let result = crate::program_stubs::arch_set_transaction_to_sign(
        serialized_transaction_to_sign.as_ptr(),
        serialized_transaction_to_sign.len(),
    );

    match result {
        crate::entrypoint::SUCCESS => {
            let tx: Transaction = bitcoin::consensus::deserialize(transaction_to_sign.tx_bytes)
                .expect("failed to deserialize tx_bytes");
            for input in transaction_to_sign.inputs_to_sign {
                if let Some(account) = accounts.iter().find(|account| *account.key == input.signer)
                {
                    account.set_utxo(&UtxoMeta::from(
                        hex::decode(tx.txid().to_string())
                            .expect("failed to decode_hex")
                            .try_into()
                            .expect("failed to try_into"),
                        input.index,
                    ));
                }
            }
            Ok(())
        }
        _ => Err(result.into()),
    }
}

/// Maximum size that can be set using [`set_return_data`].
pub const MAX_RETURN_DATA: usize = 1024;

/// Set the running program's return data.
///
/// Return data is a dedicated per-transaction buffer for data passed
/// from cross-program invoked programs back to their caller.
///
/// The maximum size of return data is [`MAX_RETURN_DATA`]. Return data is
/// retrieved by the caller with [`get_return_data`].
pub fn set_return_data(data: &[u8]) {
    unsafe { crate::syscalls::sol_set_return_data(data.as_ptr(), data.len() as u64) };
}

/// Get the return data from an invoked program.
///
/// For every transaction there is a single buffer with maximum length
/// [`MAX_RETURN_DATA`], paired with a [`Pubkey`] representing the program ID of
/// the program that most recently set the return data. Thus the return data is
/// a global resource and care must be taken to ensure that it represents what
/// is expected: called programs are free to set or not set the return data; and
/// the return data may represent values set by programs multiple calls down the
/// call stack, depending on the circumstances of transaction execution.
///
/// Return data is set by the callee with [`set_return_data`].
///
/// Return data is cleared before every CPI invocation &mdash; a program that
/// has invoked no other programs can expect the return data to be `None`; if no
/// return data was set by the previous CPI invocation, then this function
/// returns `None`.
///
/// Return data is not cleared after returning from CPI invocations &mdash; a
/// program that has called another program may retrieve return data that was
/// not set by the called program, but instead set by a program further down the
/// call stack; or, if a program calls itself recursively, it is possible that
/// the return data was not set by the immediate call to that program, but by a
/// subsequent recursive call to that program. Likewise, an external RPC caller
/// may see return data that was not set by the program it is directly calling,
/// but by a program that program called.
///
/// For more about return data see the [documentation for the return data proposal][rdp].
///
/// [rdp]: https://docs.solanalabs.com/proposals/return-data
pub fn get_return_data() -> Option<(Pubkey, Vec<u8>)> {
    use std::cmp::min;

    let mut buf = [0u8; MAX_RETURN_DATA];
    let mut program_id = Pubkey::default();

    let size = unsafe {
        crate::syscalls::sol_get_return_data(buf.as_mut_ptr(), buf.len() as u64, &mut program_id)
    };

    if size == 0 {
        None
    } else {
        let size = min(size as usize, MAX_RETURN_DATA);
        Some((program_id, buf[..size as usize].to_vec()))
    }
}

pub fn get_bitcoin_tx(txid: [u8; 32]) -> Option<Vec<u8>> {
    use std::cmp::min;

    let mut buf = [0u8; MAX_BTC_TX_SIZE];

    #[cfg(target_os = "solana")]
    let size =
        unsafe { crate::syscalls::arch_get_bitcoin_tx(buf.as_mut_ptr(), buf.len() as u64, &txid) };

    #[cfg(not(target_os = "solana"))]
    let size = crate::program_stubs::arch_get_bitcoin_tx(buf.as_mut_ptr(), buf.len(), &txid);

    if size == 0 {
        None
    } else {
        let size = min(size as usize, MAX_BTC_TX_SIZE);
        Some(buf[..size as usize].to_vec())
    }
}

pub fn get_network_xonly_pubkey() -> [u8; 32] {
    let mut buf = [0u8; 32];
    let _ = unsafe { crate::syscalls::arch_get_network_xonly_pubkey(buf.as_mut_ptr()) };
    buf
}

pub fn validate_utxo_ownership(utxo: &UtxoMeta, owner: &Pubkey) -> bool {
    #[cfg(target_os = "solana")]
    unsafe {
        crate::syscalls::arch_validate_utxo_ownership(utxo, owner) != 0
    }

    #[cfg(not(target_os = "solana"))]
    {
        crate::program_stubs::arch_validate_utxo_ownership(utxo, owner) != 0
    }
}
pub fn get_account_script_pubkey(pubkey: &Pubkey) -> [u8; 34] {
    let mut buf = [0u8; 34];

    #[cfg(target_os = "solana")]
    let _ = unsafe { crate::syscalls::arch_get_account_script_pubkey(buf.as_mut_ptr(), pubkey) };

    #[cfg(not(target_os = "solana"))]
    crate::program_stubs::arch_get_account_script_pubkey(&mut buf, pubkey);
    buf
}

pub fn get_bitcoin_block_height() -> u64 {
    unsafe { crate::syscalls::arch_get_bitcoin_block_height() }
}

pub fn get_clock() -> Clock {
    let mut clock = Clock::default();
    unsafe { crate::syscalls::arch_get_clock(&mut clock) };
    clock
}
