//! Implementations of syscalls used when `arch-program` is built for non-SBF targets.

#![cfg(not(target_os = "solana"))]
#![allow(dead_code)]

pub const UNIMPLEMENTED: u64 = 0;
use crate::{
    account::AccountInfo, entrypoint::ProgramResult, instruction::Instruction, pubkey::Pubkey,
    utxo::UtxoMeta,
};

pub(crate) fn sol_log(message: &str) {
    println!("{message}");
}
pub(crate) fn sol_log_64_(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) {
    sol_log(&format!("{arg1:?}, {arg2:?},{arg3:?},{arg4:?},{arg5:?}"))
}
pub(crate) fn sol_set_return_data(_data: *const u8, _length: u64) {
    sol_log("UNAVAILABLE");
}
pub(crate) fn sol_log_pubkey(_pubkey_addr: *const u8) {
    sol_log("UNAVAILABLE");
}
pub(crate) fn sol_log_data(_data: *const u8, _data_len: u64) {
    sol_log("UNAVAILABLE");
}
pub(crate) fn sol_get_return_data(_data: *mut u8, _length: u64, _program_id: *mut Pubkey) -> u64 {
    sol_log("UNAVAILABLE");
    UNIMPLEMENTED
}
pub(crate) fn arch_set_transaction_to_sign(_transaction_to_sign: *const u8, _length: usize) -> u64 {
    sol_log("UNAVAILABLE");
    UNIMPLEMENTED
}
pub(crate) fn arch_get_bitcoin_tx(_buf: *const u8, _buf_len: usize, _txid: &[u8; 32]) -> u64 {
    sol_log("UNAVAILABLE");
    UNIMPLEMENTED
}
pub(crate) fn arch_get_network_xonly_pubkey(_data: *mut u8) -> u64 {
    sol_log("UNAVAILABLE");
    UNIMPLEMENTED
}
pub(crate) fn arch_validate_utxo_ownership(_utxo: *const UtxoMeta, _owner: *const Pubkey) -> u64 {
    sol_log("UNAVAILABLE");
    UNIMPLEMENTED
}
pub(crate) fn arch_get_account_script_pubkey(_buf: &mut [u8; 34], _pubkey: &Pubkey) {}

pub(crate) fn sol_invoke_signed_rust(
    _instruction_addr: &Instruction,
    _account_infos: &[AccountInfo],
) -> ProgramResult {
    sol_log("SyscallStubs: sol_invoke_signed() not available");
    Ok(())
}
