use anyhow::{anyhow, Result};
use arch_program::message::Message;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sha256::digest;

use crate::signature::Signature;

pub const RUNTIME_TX_SIZE_LIMIT: usize = 10240;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct RuntimeTransaction {
    pub version: u32,
    pub signatures: Vec<Signature>,
    pub message: Message,
}

impl RuntimeTransaction {
    pub fn txid(&self) -> String {
        digest(digest(self.serialize()))
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut serilized = vec![];

        serilized.extend(self.version.to_le_bytes());
        serilized.push(self.signatures.len() as u8);
        for signature in self.signatures.iter() {
            serilized.extend(&signature.serialize());
        }
        serilized.extend(self.message.serialize());

        serilized
    }

    pub fn from_slice(data: &[u8]) -> Result<Self> {
        let mut size = 4;
        let signatures_len = data[size] as usize;
        size += 1;
        let mut signatures = Vec::with_capacity(data[size] as usize);

        for _ in 0..signatures_len {
            signatures.push(Signature::from_slice(&data[size..(size + 64)]));
            size += 64;
        }
        let message = Message::from_slice(&data[size..]);

        Ok(Self {
            version: u32::from_le_bytes(data[..4].try_into().unwrap()),
            signatures,
            message,
        })
    }

    pub fn hash(&self) -> String {
        digest(digest(self.serialize()))
    }

    pub fn check_tx_size_limit(&self) -> Result<()> {
        let serialized_tx = self.serialize();
        if serialized_tx.len() > RUNTIME_TX_SIZE_LIMIT {
            Err(anyhow!(format!(
                "runtime tx size exceeds RUNTIME_TX_SIZE_LIMIT {} {}",
                serialized_tx.len(),
                RUNTIME_TX_SIZE_LIMIT
            )))
        } else {
            Ok(())
        }
    }
}

use arch_program::instruction::Instruction;
use arch_program::pubkey::Pubkey;
use proptest::prelude::*;

proptest! {
    #[test]
    fn fuzz_serialize_deserialize_runtime_transaction(
        version in any::<u32>(),
        signatures in prop::collection::vec(prop::collection::vec(any::<u8>(), 64), 0..10),
        signers in prop::collection::vec(any::<[u8; 32]>(), 0..10),
        instructions in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..100), 0..10)
    ) {
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

        let transaction = RuntimeTransaction {
            version,
            signatures,
            message,
        };

        let serialized = transaction.serialize();
        let deserialized = RuntimeTransaction::from_slice(&serialized).unwrap();
        assert_eq!(transaction, deserialized);
    }
}
