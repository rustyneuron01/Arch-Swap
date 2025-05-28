use crate::account::AccountMeta;
use crate::instruction::Instruction;
use crate::pubkey::Pubkey;
use crate::utxo::UtxoMeta;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SystemInstruction {
    CreateAccount(UtxoMeta),
    ExtendBytes(Vec<u8>),
}

impl SystemInstruction {
    pub fn serialise(&self) -> Vec<u8> {
        let mut serialized = vec![];

        match self {
            Self::CreateAccount(utxo) => {
                serialized.push(0);
                serialized.extend(utxo.serialize());
            }
            Self::ExtendBytes(bytes) => {
                serialized.push(1);
                serialized.extend(bytes);
            }
        }

        serialized
    }

    pub fn from_slice(data: &[u8]) -> Self {
        match data[0] {
            0 => Self::CreateAccount(UtxoMeta::from_slice(&data[1..])),
            1 => Self::ExtendBytes(data[1..].to_vec()),
            _ => {
                unreachable!("error deserializing system instruction")
            }
        }
    }

    pub fn new_create_account_instruction(
        txid: [u8; 32],
        vout: u32,
        pubkey: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id: Pubkey::system_program(),
            accounts: vec![AccountMeta {
                pubkey,
                is_signer: true,
                is_writable: true,
            }],
            data: SystemInstruction::CreateAccount(UtxoMeta::from(txid, vout)).serialise(),
        }
    }

    pub fn new_extend_bytes_instruction(data: Vec<u8>, pubkey: Pubkey) -> Instruction {
        Instruction {
            program_id: Pubkey::system_program(),
            accounts: vec![AccountMeta {
                pubkey,
                is_signer: true,
                is_writable: true,
            }],
            data: SystemInstruction::ExtendBytes(data).serialise(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SystemInstruction;
    use crate::utxo::UtxoMeta;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fuzz_serialize_deserialize_system_instruction_create_account(
            txid in any::<[u8; 32]>(),
            vout in any::<u32>(),
            random_bytes in prop::collection::vec(any::<u8>(), 0..1024),
        ) {
            let instruction = SystemInstruction::CreateAccount(UtxoMeta::from(txid, vout));

            let serialized = instruction.serialise();
            let deserialized = SystemInstruction::from_slice(&serialized);

            assert_eq!(instruction, deserialized);
        }

        #[test]
        fn fuzz_serialize_deserialize_system_instruction_extend_bytes(
            txid in any::<[u8; 32]>(),
            vout in any::<u32>(),
            random_bytes in prop::collection::vec(any::<u8>(), 0..1024),
        ) {
            let instruction = SystemInstruction::ExtendBytes(random_bytes.clone());

            let serialized = instruction.serialise();
            let deserialized = SystemInstruction::from_slice(&serialized);

            assert_eq!(instruction, deserialized);
        }
    }
}
