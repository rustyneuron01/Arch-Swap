//! This module represents states for the running processes

use crate::signature::Signature;
use anyhow::{anyhow, Result};
use arch_program::pubkey::Pubkey;
use bitcoin::{
    self,
    address::Address,
    key::{Parity, UntweakedKeypair, XOnlyPublicKey},
    secp256k1::{Secp256k1, SecretKey},
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use sha256::digest;
use std::fs;
use std::str::FromStr;

/// Represents the parameters for deploying a program
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeployProgramParams {
    pub elf: Vec<u8>,
}

/// Represents the parameters for reading a utxo
#[derive(Serialize, Deserialize, Debug)]
pub struct ReadUtxoParams {
    pub utxo_id: String,
}

pub struct BitcoinRpcInfo {
    pub endpoint: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorityMessage {
    pub utxo: Utxo,
    pub data: Vec<u8>,
    pub authority: Pubkey,
}

impl AuthorityMessage {
    pub fn hash(&self) -> Result<String> {
        Ok(digest(digest(match to_string(self) {
            Ok(d) => d,
            Err(err) => return Err(anyhow!("{:?}", err)),
        })))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssignAuthorityParams {
    pub signature: Signature,
    pub message: AuthorityMessage,
}

/// Represents a party or node secret and address information
pub struct CallerInfo {
    pub key_pair: UntweakedKeypair,
    pub public_key: XOnlyPublicKey,
    pub parity: Parity,
    pub address: Address,
}

impl CallerInfo {
    /// Create a [CallerInfo] from the specified file path
    /// If the file does not exist, generate a random secret key
    /// and use that instead.
    pub fn with_secret_key_file(file_path: &str) -> Result<CallerInfo> {
        let secp = Secp256k1::new();
        let secret_key = match fs::read_to_string(file_path) {
            Ok(key) => SecretKey::from_str(&key).unwrap(),
            Err(_) => {
                let (key, _) = secp.generate_keypair(&mut OsRng);
                fs::write(file_path, key.display_secret().to_string())
                    .map_err(|_| anyhow!("Unable to write file"))?;
                key
            }
        };
        let key_pair = UntweakedKeypair::from_secret_key(&secp, &secret_key);
        let (public_key, parity) = XOnlyPublicKey::from_keypair(&key_pair);
        let address = Address::p2tr(&secp, public_key, None, bitcoin::Network::Regtest);
        Ok(CallerInfo {
            key_pair,
            public_key,
            parity,
            address,
        })
    }
}
