#![allow(improper_ctypes)]

use crate::{clock::Clock, pubkey::Pubkey, utxo::UtxoMeta};

macro_rules! define_syscall {
	(fn $name:ident($($arg:ident: $typ:ty),*) -> $ret:ty) => {
		extern "C" {
			pub fn $name($($arg: $typ),*) -> $ret;
		}
	};
	(fn $name:ident($($arg:ident: $typ:ty),*)) => {
		define_syscall!(fn $name($($arg: $typ),*) -> ());
	}
}

define_syscall!(fn sol_invoke_signed_rust(instruction_addr: *const u8, account_infos_addr: *const u8, account_infos_len: u64) -> u64);
define_syscall!(fn sol_set_return_data(data: *const u8, length: u64));
define_syscall!(fn sol_get_return_data(data: *mut u8, length: u64, program_id: *mut Pubkey) -> u64);

define_syscall!(fn arch_set_transaction_to_sign(transaction_to_sign: *const u8, length: u64) -> u64);
define_syscall!(fn arch_get_bitcoin_tx(data: *mut u8, length: u64, txid: &[u8; 32]) -> u64);
define_syscall!(fn arch_get_network_xonly_pubkey(data: *mut u8) -> u64);
define_syscall!(fn arch_validate_utxo_ownership(utxo: *const UtxoMeta, owner: *const Pubkey) -> u64);
define_syscall!(fn arch_get_account_script_pubkey(script: *mut u8, pubkey: *const Pubkey) -> u64);
define_syscall!(fn arch_get_bitcoin_block_height() -> u64);
define_syscall!(fn arch_get_clock(clock: *mut Clock) -> u64);
// logs
define_syscall!(fn sol_log_(message: *const u8, len: u64));
define_syscall!(fn sol_log_64_(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64));
define_syscall!(fn sol_log_pubkey(pubkey_addr: *const u8));
define_syscall!(fn sol_log_data(data: *const u8, data_len: u64));
