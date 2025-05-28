use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::runtime_transaction::RuntimeTransaction;

#[derive(Clone, Debug, Deserialize, Serialize, BorshDeserialize, BorshSerialize, PartialEq)]
pub enum Status {
    Processing,
    Processed,
    Failed(String),
}
impl Status {
    pub fn from_value(value: &Value) -> Option<Self> {
        if let Some(status_str) = value.as_str() {
            match status_str {
                "Processing" => return Some(Status::Processing),
                _ => return Some(Status::Processed),
            }
        } else if let Some(obj) = value.as_object() {
            if let Some(failed_message) = obj.get("Failed").and_then(|v| v.as_str()) {
                return Some(Status::Failed(failed_message.to_string()));
            } else {
                return None;
            }
        }
        None
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ProcessedTransaction {
    pub runtime_transaction: RuntimeTransaction,
    pub status: Status,
    pub bitcoin_txid: Option<String>,
    pub accounts_tags: Vec<String>,
}

impl ProcessedTransaction {
    pub fn txid(&self) -> String {
        self.runtime_transaction.txid()
    }

    pub fn to_vec(&self) -> Result<Vec<u8>> {
        let mut serialized = vec![];

        serialized.extend((self.runtime_transaction.serialize().len() as u64).to_le_bytes());
        serialized.extend(self.runtime_transaction.serialize());

        serialized.extend(match &self.bitcoin_txid {
            Some(txid) => {
                let mut res = vec![1];
                res.extend(hex::decode(txid)?);
                res
            }
            None => vec![0],
        });

        serialized.extend((self.accounts_tags.len() as u64).to_le_bytes());
        for account_tag in &self.accounts_tags {
            serialized.extend(hex::decode(account_tag)?);
        }

        serialized.extend(match &self.status {
            Status::Processing => vec![0_u8],
            Status::Processed => vec![1_u8],
            Status::Failed(err) => {
                let mut result = vec![2_u8];
                result.extend((err.len() as u64).to_le_bytes());
                result.extend(err.as_bytes());
                result
            }
        });
        Ok(serialized)
    }

    pub fn from_vec(data: &[u8]) -> Result<Self> {
        let data_bytes = data[..8].try_into()?;
        let runtime_transaction_len = u64::from_le_bytes(data_bytes) as usize;
        let mut size = 8;
        let runtime_transaction =
            RuntimeTransaction::from_slice(&data[size..(size + runtime_transaction_len)])?;
        size += runtime_transaction_len;

        let bitcoin_txid = if data[size] == 1 {
            size += 1;
            let res = Some(hex::encode(&data[(size)..(size + 32)]));
            size += 32;
            res
        } else {
            size += 1;
            None
        };

        let data_bytes = data[size..(size + 8)].try_into()?;
        let accounts_tags_len = u64::from_le_bytes(data_bytes) as usize;
        size += 8;
        let mut accounts_tags = vec![];
        for _ in 0..accounts_tags_len {
            accounts_tags.push(hex::encode(&data[(size)..(size + 32)]));
            size += 32;
        }

        let status = match data[size] {
            0 => Status::Processing,
            1 => Status::Processed,
            2 => {
                let data_bytes = data[(size + 1)..(size + 9)].try_into()?;
                let error_len = u64::from_le_bytes(data_bytes) as usize;
                size += 9;
                let error = String::from_utf8(data[size..(size + error_len)].to_vec())?;
                Status::Failed(error)
            }
            _ => unreachable!("status doesn't exist"),
        };

        Ok(ProcessedTransaction {
            runtime_transaction,
            status,
            bitcoin_txid,
            accounts_tags,
        })
    }
}

use crate::signature::Signature;
use arch_program::instruction::Instruction;
use arch_program::message::Message;
use arch_program::pubkey::Pubkey;
use proptest::prelude::*;
use proptest::strategy::Just;

proptest! {
    #[test]
    fn fuzz_serialize_deserialize_processed_transaction(
        version in any::<u32>(),
        signatures in prop::collection::vec(prop::collection::vec(any::<u8>(), 64), 0..10),
        signers in prop::collection::vec(any::<[u8; 32]>(), 0..10),
        instructions in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..100), 0..10),
        bitcoin_txid in "[0-9a-f]{64}",
        accounts_tags in prop::collection::vec("[0-9a-f]{64}", 0..10)
    ) {
        // Generate a random RuntimeTransaction
        let signatures: Vec<Signature> = signatures.into_iter()
            .map(|sig_bytes| Signature::from_slice(&sig_bytes))
            .collect();

        let signers: Vec<Pubkey> = signers.into_iter()
            .map(Pubkey::from)
            .collect();

        let instructions: Vec<Instruction> = instructions.into_iter()
            .map(|data| Instruction {
                program_id: Pubkey::system_program(),
                accounts: vec![],
                data,
            })
            .collect();

        let message = Message {
            signers,
            instructions,
        };

        let runtime_transaction = RuntimeTransaction {
            version,
            signatures,
            message,
        };

        let processed_transaction = ProcessedTransaction {
            runtime_transaction,
            status: Status::Processing,
            bitcoin_txid: Some(bitcoin_txid.to_string()),
            accounts_tags: accounts_tags.iter().map(|s| s.to_string()).collect(),
        };

        let serialized = processed_transaction.to_vec().unwrap();
        let deserialized = ProcessedTransaction::from_vec(&serialized).unwrap();

        let reserialized = deserialized.to_vec().unwrap();
        assert_eq!(serialized, reserialized);
    }
}
