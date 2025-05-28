use crate::pubkey::Pubkey;

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub struct InputToSign {
    pub index: u32,
    pub signer: Pubkey,
}
