use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Signature(pub Vec<u8>);

impl Signature {
    pub fn serialize(&self) -> Vec<u8> {
        self.0.clone()
    }

    pub fn from_slice(data: &[u8]) -> Self {
        Self(data[..64].to_vec())
    }
}

use proptest::prelude::*;

proptest! {
    #[test]
    fn fuzz_serialize_deserialize_signature(signature_bytes in prop::collection::vec(any::<u8>(), 64..128)) {
        let signature = Signature::from_slice(&signature_bytes);
        let serialized = signature.serialize();
        let deserialized = Signature::from_slice(&serialized);
        assert_eq!(signature, deserialized);
    }
}
