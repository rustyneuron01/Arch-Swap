//! This module contains constants

use bitcoin::Network;

/// The file path where the caller stores information
pub const CALLER_FILE_PATH: &str = "src/app/keys/caller.json";
pub const PROGRAM_FILE_PATH: &str = "src/app/keys/program.json";

/// Local address for node 1
pub const NODE1_ADDRESS: &str = "http://127.0.0.1:9002/";

/// Arbitrary example names for HelloWorld program
pub const NAME1: &str = "Amine";
pub const NAME2: &str = "Marouane";

/// RPC methods
pub const ASSIGN_AUTHORITY: &str = "assign_authority";
pub const READ_ACCOUNT_INFO: &str = "read_account_info";
pub const DEPLOY_PROGRAM: &str = "deploy_program";
pub const SEND_TRANSACTION: &str = "send_transaction";
pub const GET_PROGRAM: &str = "get_program";
pub const GET_BLOCK: &str = "get_block";
pub const GET_BEST_BLOCK_HASH: &str = "get_best_block_hash";
pub const GET_PROCESSED_TRANSACTION: &str = "get_processed_transaction";
pub const GET_ACCOUNT_ADDRESS: &str = "get_account_address";

/// Data
pub const BITCOIN_NODE_ENDPOINT: &str = "http://localhost:18443/wallet/regtest_desc_wallet";
pub const BITCOIN_NODE_USERNAME: &str = "user";
pub const BITCOIN_NODE_PASSWORD: &str = "password";

// Specify which Bitcoin network
pub const BITCOIN_NETWORK: Network = Network::Regtest;

/// Hack for Error codes
pub const TRANSACTION_NOT_FOUND_CODE: i64 = 404;
