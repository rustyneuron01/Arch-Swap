//! This module contains helper methods for interacting with the HelloWorld program

use anyhow::{anyhow, Result};
use arch_program::instruction::Instruction;
use bip322::sign_simple;
use bitcoin::{
    absolute::LockTime,
    address::Address,
    key::{TapTweak, TweakedKeypair},
    secp256k1::{self, Secp256k1},
    sighash::{Prevouts, SighashCache},
    transaction::Version,
    Amount, OutPoint, PrivateKey, ScriptBuf, Sequence, TapSighashType, Transaction, TxIn, Txid,
    Witness,
};
use bitcoincore_rpc::{Auth, Client, RawTx, RpcApi};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use serde::Serialize;
use serde_json::{from_str, json, Value};
use std::fs;
use std::str::FromStr;
use tokio::task;

use crate::processed_transaction::ProcessedTransaction;

use crate::constants::{
    BITCOIN_NODE_ENDPOINT, BITCOIN_NODE_PASSWORD, BITCOIN_NODE_USERNAME, CALLER_FILE_PATH,
    GET_ACCOUNT_ADDRESS, GET_BEST_BLOCK_HASH, GET_BLOCK, GET_PROCESSED_TRANSACTION, GET_PROGRAM,
    NODE1_ADDRESS, READ_ACCOUNT_INFO, TRANSACTION_NOT_FOUND_CODE,
};
use crate::helper::secp256k1::SecretKey;
use crate::models::CallerInfo;
use crate::runtime_transaction::RuntimeTransaction;
use crate::signature::Signature;
use arch_program::message::Message;
use arch_program::pubkey::Pubkey;
use bitcoin::key::UntweakedKeypair;
use bitcoin::XOnlyPublicKey;
use rand_core::OsRng;

pub fn process_result(response: String) -> Result<Value> {
    let result = from_str::<Value>(&response).expect("result should be Value parseable");

    let result = match result {
        Value::Object(object) => object,
        _ => panic!("unexpected output"),
    };

    if let Some(err) = result.get("error") {
        return Err(anyhow!("{:?}", err));
    }

    Ok(result["result"].clone())
}

pub fn process_get_transaction_result(response: String) -> Result<Value> {
    let result = from_str::<Value>(&response).expect("result should be string parseable");

    let result = match result {
        Value::Object(object) => object,
        _ => panic!("unexpected output"),
    };

    if let Some(err) = result.get("error") {
        if let Value::Number(code) = result["error"]["code"].clone() {
            if code.as_i64() == Some(TRANSACTION_NOT_FOUND_CODE) {
                return Ok(Value::Null);
            }
        }
        return Err(anyhow!("{:?}", err));
    }

    Ok(result["result"].clone())
}

pub fn post(url: &str, method: &str) -> String {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(url)
        .header("content-type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "curlycurl",
            "method": method,
        }))
        .send()
        .expect("post method should not fail");

    res.text().expect("result should be text decodable")
}

pub fn post_data<T: Serialize + std::fmt::Debug>(url: &str, method: &str, params: T) -> String {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(url)
        .header("content-type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "curlycurl",
            "method": method,
            "params": params,
        }))
        .send();

    res.expect("post method should not fail")
        .text()
        .expect("result should be text decodable")
}

/// Returns a caller information using the secret key file specified
fn _get_trader(trader_id: u64) -> Result<CallerInfo> {
    let file_path = &format!("../../.arch/trader{}.json", trader_id);
    CallerInfo::with_secret_key_file(file_path)
}

pub fn with_secret_key_file(file_path: &str) -> Result<(UntweakedKeypair, Pubkey)> {
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
    let keypair = UntweakedKeypair::from_secret_key(&secp, &secret_key);
    let pubkey = Pubkey::from_slice(&XOnlyPublicKey::from_keypair(&keypair).0.serialize());
    Ok((keypair, pubkey))
}

use crate::runtime_transaction::RUNTIME_TX_SIZE_LIMIT;
use arch_program::system_instruction::SystemInstruction;

pub fn extend_bytes_max_len() -> usize {
    let message = Message {
        signers: vec![Pubkey::system_program()],
        instructions: vec![SystemInstruction::new_extend_bytes_instruction(
            vec![0_u8; 8],
            Pubkey::system_program(),
        )],
    };

    RUNTIME_TX_SIZE_LIMIT
        - RuntimeTransaction {
            version: 0,
            signatures: vec![Signature([0_u8; 64].to_vec())],
            message,
        }
        .serialize()
        .len()
}

pub fn sign_message_bip322(keypair: &UntweakedKeypair, msg: &[u8]) -> [u8; 64] {
    let secp = Secp256k1::new();
    let xpubk = XOnlyPublicKey::from_keypair(keypair).0;
    let private_key = PrivateKey::new(SecretKey::from_keypair(keypair), bitcoin::Network::Regtest);

    let address = Address::p2tr(&secp, xpubk, None, bitcoin::Network::Regtest);
    let signature = sign_simple(&address, msg, private_key).unwrap();

    signature.to_vec()[0][..64].try_into().unwrap()
}

/// Creates an instruction, signs it as a message
/// and sends the signed message as a transaction
pub fn sign_and_send_instruction(
    instruction: Instruction,
    signers: Vec<UntweakedKeypair>,
) -> Result<(String, String)> {
    // Get public keys from signers
    let pubkeys = signers
        .iter()
        .map(|signer| {
            let pubkey = Pubkey::from_slice(&XOnlyPublicKey::from_keypair(signer).0.serialize());
            pubkey
        })
        .collect::<Vec<Pubkey>>();

    // Step 2: Create a message with the instruction and signers
    let message = Message {
        signers: pubkeys.clone(), // Clone for logging purposes
        instructions: vec![instruction.clone()],
    };

    // Step 3: Hash the message and decode
    let message_hash = message.hash();
    let digest_slice = hex::decode(message_hash).expect("hashed message should be decodable");

    // Step 5: Sign the message with each signer's key
    let signatures = signers
        .iter()
        .map(|signer| {
            let signature = sign_message_bip322(signer, &digest_slice).to_vec();
            Signature(signature)
        })
        .collect::<Vec<Signature>>();

    //println!("Message signed by {} signers",signatures.len());

    // Step 6: Create transaction parameters
    let params = RuntimeTransaction {
        version: 0,
        signatures: signatures.clone(), // Clone for logging purposes
        message: message.clone(),       // Clone for logging purposes
    };

    //println!("Runtime Transaction constructed : {:?} ",params);
    // Step 7: Send transaction to node for processeing
    let result = process_result(post_data(NODE1_ADDRESS, "send_transaction", params))
        .expect("send_transaction should not fail")
        .as_str()
        .expect("cannot convert result to string")
        .to_string();

    //println!("Arch transaction ID: {:?}", result);

    // Step 8: Hash the instruction
    let hashed_instruction = instruction.hash();

    Ok((result, hashed_instruction))
}

pub fn sign_and_send_transaction(
    instructions: Vec<Instruction>,
    signers: Vec<UntweakedKeypair>,
) -> Result<String> {
    let pubkeys = signers
        .iter()
        .map(|signer| Pubkey::from_slice(&XOnlyPublicKey::from_keypair(signer).0.serialize()))
        .collect::<Vec<Pubkey>>();

    let message = Message {
        signers: pubkeys,
        instructions,
    };
    let digest_slice = hex::decode(message.hash()).expect("hashed message should be decodable");
    let signatures = signers
        .iter()
        .map(|signer| Signature(sign_message_bip322(signer, &digest_slice).to_vec()))
        .collect::<Vec<Signature>>();

    let params = RuntimeTransaction {
        version: 0,
        signatures,
        message,
    };
    let result = process_result(post_data(NODE1_ADDRESS, "send_transaction", params))
        .expect("send_transaction should not fail")
        .as_str()
        .expect("cannot convert result to string")
        .to_string();

    Ok(result)
}

/// Deploys the HelloWorld program using the compiled ELF
pub fn deploy_program_txs(program_keypair: UntweakedKeypair, elf_path: &str) -> Result<()> {
    let program_pubkey =
        Pubkey::from_slice(&XOnlyPublicKey::from_keypair(&program_keypair).0.serialize());

    let elf = fs::read(elf_path).expect("elf path should be available");

    let txs = elf
        .chunks(extend_bytes_max_len())
        .enumerate()
        .map(|(i, chunk)| {
            let mut bytes = vec![];

            let offset: u32 = (i * extend_bytes_max_len()) as u32;
            let len: u32 = chunk.len() as u32;

            bytes.extend(offset.to_le_bytes());
            bytes.extend(len.to_le_bytes());
            bytes.extend(chunk);

            let message = Message {
                signers: vec![program_pubkey],
                instructions: vec![SystemInstruction::new_extend_bytes_instruction(
                    bytes,
                    program_pubkey,
                )],
            };

            let digest_slice =
                hex::decode(message.hash()).expect("hashed message should be decodable");

            RuntimeTransaction {
                version: 0,
                signatures: vec![Signature(
                    sign_message_bip322(&program_keypair, &digest_slice).to_vec(),
                )],
                message,
            }
        })
        .collect::<Vec<RuntimeTransaction>>();

    let response = post_data(
        &NODE1_ADDRESS.to_string(),
        &"send_transactions".to_string(),
        txs.clone(),
    );

    let txids = process_result(response)?
        .as_array()
        .ok_or_else(|| anyhow!("Invalid response format"))?
        .iter()
        .map(|r| r.as_str().unwrap_or_default().to_string())
        .collect::<Vec<String>>();

    let pb = ProgressBar::new(txids.len() as u64);

    pb.set_style(ProgressStyle::default_bar()
        .progress_chars("#>-")
        .template("{spinner:.green}[{elapsed_precise:.blue}] {msg:.blue} [{bar:100.green/blue}] {pos}/{len} ({eta})").unwrap());

    pb.set_message("Successfully Processed Deployment Transactions :");

    for txid in txids {
        let processed_tx = get_processed_transaction(NODE1_ADDRESS, txid.clone())
            .expect("get processed transaction should not fail");
        pb.inc(1);
        pb.set_message("Successfully Processed Deployment Transactions :");
    }

    pb.finish();

    Ok(())
}
async fn async_post_data<T: Serialize + std::fmt::Debug + Send + 'static>(
    url: String,
    method: String,
    params: T,
) -> Result<String> {
    Ok(task::spawn_blocking(move || post_data(&url, &method, params)).await?)
}

/// Starts Key Exchange by calling the RPC method
pub fn start_key_exchange() {
    match process_result(post(NODE1_ADDRESS, "start_key_exchange")) {
        Err(err) => println!("Error starting Key Exchange: {:?}", err),
        Ok(val) => assert!(val.as_bool().unwrap()),
    };
}

/// Starts a Distributed Key Generation round by calling the RPC method
pub fn start_dkg() {
    if let Err(err) = process_result(post(NODE1_ADDRESS, "start_dkg")) {
        println!("Error starting DKG: {:?}", err);
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountInfoResult {
    pub owner: Pubkey,
    pub data: Vec<u8>,
    pub utxo: String,
    pub is_executable: bool,
    pub tag: String,
}

/// Read Utxo given the utxo ID
pub fn read_account_info(url: &str, pubkey: Pubkey) -> Result<AccountInfoResult> {
    let result = process_result(post_data(url, READ_ACCOUNT_INFO, pubkey))?;
    serde_json::from_value(result).map_err(|_| anyhow!("Unable to decode read_account_info result"))
}
/*
pub async fn get_program_accounts(
    context: Arc<ValidatorContext>,
    program_id: Pubkey,
    filters: Option<Vec<AccountFilter>>,
) -> Result<Vec<ProgramAccount>, ErrorObject<'static>> {
    match context
        .rocks_db()
        .await
        .get_program_accounts(&program_id, filters)
    {
        Ok(accounts) => Ok(accounts),
        Err(err) => {
            error!("Error fetching program accounts: {:?}", err);
            Err(ErrorObject::borrowed(
                ErrorCode::InternalError.code(),
                "Error fetching program accounts",
                None,
            ))
        }
    }
}
*/

/// Returns a program given the program ID
pub fn get_program(url: &str, program_id: String) -> String {
    process_result(post_data(url, GET_PROGRAM, program_id))
        .expect("get_program should not fail")
        .as_str()
        .expect("cannot convert result to string")
        .to_string()
}

/// Returns the best block
fn _get_best_block() -> String {
    let best_block_hash = process_result(post(NODE1_ADDRESS, GET_BEST_BLOCK_HASH))
        .expect("best_block_hash should not fail")
        .as_str()
        .expect("cannot convert result to string")
        .to_string();
    process_result(post_data(NODE1_ADDRESS, GET_BLOCK, best_block_hash))
        .expect("get_block should not fail")
        .as_str()
        .expect("cannot convert result to string")
        .to_string()
}

/// Returns a processed transaction given the txid
/// Keeps trying for a maximum of 60 seconds if the processed transaction is not available
pub fn get_processed_transaction<'a>(url: &str, tx_id: String) -> Result<ProcessedTransaction> {
    let mut processed_tx =
        process_get_transaction_result(post_data(url, GET_PROCESSED_TRANSACTION, tx_id.clone()));
    if let Err(e) = processed_tx {
        return Err(anyhow!("{}", e));
    }

    let mut wait_time = 1;
    while let Ok(Value::Null) = processed_tx {
        std::thread::sleep(std::time::Duration::from_secs(wait_time));
        processed_tx = process_get_transaction_result(post_data(
            url,
            GET_PROCESSED_TRANSACTION,
            tx_id.clone(),
        ));
        wait_time += 1;
        if wait_time >= 60 {
            println!("get_processed_transaction has run for more than 60 seconds");
            return Err(anyhow!("Failed to retrieve processed transaction"));
        }
    }

    if let Ok(ref tx) = processed_tx {
        let mut p = tx.clone();

        let get_status = |p: Value| -> String {
            if p["status"].as_str().is_some() {
                p["status"].as_str().unwrap().to_string()
            } else if let Some(val) = p["status"].as_object() {
                if val.contains_key("Failed") {
                    "Failed".to_string()
                } else {
                    unreachable!("WTFFF");
                }
            } else {
                unreachable!("WTFFF2");
            }
        };

        while get_status(p.clone()) != "Processed".to_string()
            && get_status(p.clone()) != "Failed".to_string()
        {
            println!("Processed transaction is not yet finalized. Retrying...");
            std::thread::sleep(std::time::Duration::from_secs(wait_time));
            p = process_get_transaction_result(post_data(
                url,
                GET_PROCESSED_TRANSACTION,
                tx_id.clone(),
            ))
            .unwrap();
            wait_time += 10;
            if wait_time >= 60 {
                println!("get_processed_transaction has run for more than 60 seconds");
                return Err(anyhow!("Failed to retrieve processed transaction"));
            }
        }
        processed_tx = Ok(p);
    }

    Ok(serde_json::from_value(processed_tx?).unwrap())
}

pub fn prepare_fees() -> String {
    let userpass = Auth::UserPass(
        BITCOIN_NODE_USERNAME.to_string(),
        BITCOIN_NODE_PASSWORD.to_string(),
    );
    let rpc =
        Client::new(BITCOIN_NODE_ENDPOINT, userpass).expect("rpc shouldn not fail to be initiated");

    let caller = CallerInfo::with_secret_key_file(CALLER_FILE_PATH)
        .expect("getting caller info should not fail");

    let txid = rpc
        .send_to_address(
            &caller.address,
            Amount::from_sat(3000),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("SATs should be sent to address");

    let sent_tx = rpc
        .get_raw_transaction(&txid, None)
        .expect("should get raw transaction");
    let mut vout = 0;

    for (index, output) in sent_tx.output.iter().enumerate() {
        if output.script_pubkey == caller.address.script_pubkey() {
            vout = index as u32;
        }
    }

    let mut tx = Transaction {
        version: Version::TWO,
        input: vec![TxIn {
            previous_output: OutPoint { txid, vout },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
        }],
        output: vec![],
        lock_time: LockTime::ZERO,
    };

    let sighash_type = TapSighashType::NonePlusAnyoneCanPay;
    let raw_tx = rpc
        .get_raw_transaction(&txid, None)
        .expect("raw transaction should not fail");
    let prevouts = vec![raw_tx.output[vout as usize].clone()];
    let prevouts = Prevouts::All(&prevouts);

    let mut sighasher = SighashCache::new(&mut tx);
    let sighash = sighasher
        .taproot_key_spend_signature_hash(0, &prevouts, sighash_type)
        .expect("should not fail to construct sighash");

    // Sign the sighash using the secp256k1 library (exported by rust-bitcoin).
    let secp = Secp256k1::new();
    let tweaked: TweakedKeypair = caller.key_pair.tap_tweak(&secp, None);
    let msg = secp256k1::Message::from(sighash);
    let signature = secp.sign_schnorr(&msg, &tweaked.to_inner());

    // Update the witness stack.
    let signature = bitcoin::taproot::Signature {
        sig: signature,
        hash_ty: sighash_type,
    };
    tx.input[0].witness.push(signature.to_vec());

    tx.raw_hex()
}

pub fn send_utxo(pubkey: Pubkey) -> (String, u32) {
    let userpass = Auth::UserPass(
        BITCOIN_NODE_USERNAME.to_string(),
        BITCOIN_NODE_PASSWORD.to_string(),
    );
    let rpc =
        Client::new(BITCOIN_NODE_ENDPOINT, userpass).expect("rpc shouldn not fail to be initiated");

    let _caller = CallerInfo::with_secret_key_file(CALLER_FILE_PATH)
        .expect("getting caller info should not fail");

    let address = get_account_address(pubkey);

    /*println!(
        "Arch Account Address for Public key {:x} is {}",
        pubkey, address
    );*/

    let account_address = Address::from_str(&address)
        .unwrap()
        .require_network(bitcoin::Network::Regtest)
        .unwrap();

    let txid = rpc
        .send_to_address(
            &account_address,
            Amount::from_sat(3000),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("SATs should be sent to address");

    let sent_tx = rpc
        .get_raw_transaction(&txid, None)
        .expect("should get raw transaction");
    let mut vout = 0;

    for (index, output) in sent_tx.output.iter().enumerate() {
        if output.script_pubkey == account_address.script_pubkey() {
            vout = index as u32;
        }
    }

    (txid.to_string(), vout)
}

pub fn send_utxo_2(pubkey: Pubkey) -> (Txid, u32) {
    let userpass = Auth::UserPass(
        BITCOIN_NODE_USERNAME.to_string(),
        BITCOIN_NODE_PASSWORD.to_string(),
    );
    let rpc =
        Client::new(BITCOIN_NODE_ENDPOINT, userpass).expect("rpc shouldn not fail to be initiated");

    let _caller = CallerInfo::with_secret_key_file(CALLER_FILE_PATH)
        .expect("getting caller info should not fail");

    let address = get_account_address(pubkey);
    println!("address {:?}", address);
    let account_address = Address::from_str(&address)
        .unwrap()
        .require_network(bitcoin::Network::Regtest)
        .unwrap();

    let txid = rpc
        .send_to_address(
            &account_address,
            Amount::from_sat(3000),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("SATs should be sent to address");

    let sent_tx = rpc
        .get_raw_transaction(&txid, None)
        .expect("should get raw transaction");
    let mut vout = 0;

    for (index, output) in sent_tx.output.iter().enumerate() {
        if output.script_pubkey == account_address.script_pubkey() {
            vout = index as u32;
            println!("FOUUUND MATCHING UTXO")
        }
    }

    (txid, vout)
}

pub fn get_account_address(pubkey: Pubkey) -> String {
    process_result(post_data(
        NODE1_ADDRESS,
        GET_ACCOUNT_ADDRESS,
        pubkey.serialize(),
    ))
    .expect("get_account_address should not fail")
    .as_str()
    .expect("cannot convert result to string")
    .to_string()
}

fn _get_address_utxos(rpc: &Client, address: String) -> Vec<Value> {
    let client = reqwest::blocking::Client::new();

    let res = client
        .get(format!(
            "https://mempool.dev.aws.archnetwork.xyz/api/address/{}/utxo",
            address
        ))
        .header("Accept", "application/json")
        .send()
        .unwrap();

    let utxos = from_str::<Value>(&res.text().unwrap()).unwrap();

    utxos
        .as_array()
        .unwrap()
        .iter()
        .filter(|utxo| {
            utxo["status"]["block_height"].as_u64().unwrap() <= rpc.get_block_count().unwrap() - 100
        })
        .cloned()
        .collect()
}
