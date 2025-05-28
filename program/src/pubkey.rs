use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    PartialOrd,
    Ord,
    Default,
    Copy,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct Pubkey(pub [u8; 32]);

impl Pubkey {
    pub fn serialize(&self) -> [u8; 32] {
        self.0
    }

    pub fn from_slice(data: &[u8]) -> Self {
        let mut tmp = [0u8; 32];
        tmp[..data.len()].copy_from_slice(data);
        Self(tmp)
    }

    pub fn system_program() -> Self {
        let mut tmp = [0u8; 32];
        tmp[31] = 1;
        Self(tmp)
    }

    pub fn is_system_program(&self) -> bool {
        let mut tmp = [0u8; 32];
        tmp[31] = 1;
        self.0 == tmp
    }

    /// unique Pubkey for tests and benchmarks.
    pub fn new_unique() -> Self {
        use crate::atomic_u64::AtomicU64;
        static I: AtomicU64 = AtomicU64::new(1);

        let mut b = [0u8; 32];
        let i = I.fetch_add(1);
        // use big endian representation to ensure that recent unique pubkeys
        // are always greater than less recent unique pubkeys
        b[0..8].copy_from_slice(&i.to_be_bytes());
        Self::from(b)
    }

    /// Log a `Pubkey` from a program
    pub fn log(&self) {
        unsafe { crate::syscalls::sol_log_pubkey(self.as_ref() as *const _ as *const u8) };
    }
}

impl std::fmt::LowerHex for Pubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ser = self.serialize();
        for ch in &ser[..] {
            write!(f, "{:02x}", *ch)?;
        }
        Ok(())
    }
}

use core::fmt;

/// TODO:
///  Change this in future according to the correct base implementation
impl fmt::Display for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl AsRef<[u8]> for Pubkey {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl AsMut<[u8]> for Pubkey {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

impl From<[u8; 32]> for Pubkey {
    fn from(value: [u8; 32]) -> Self {
        Pubkey(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::pubkey::Pubkey;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fuzz_serialize_deserialize_pubkey(data in any::<[u8; 32]>()) {
            let pubkey = Pubkey::from(data);
            let serialized = pubkey.serialize();
            let deserialized = Pubkey::from_slice(&serialized);
            assert_eq!(pubkey, deserialized);
        }
    }
}
