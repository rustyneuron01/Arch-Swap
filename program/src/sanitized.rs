use crate::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct SanitizedMessage {
    pub signers: Vec<Pubkey>,
    pub instructions: Vec<SanitizedInstruction>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SanitizedInstruction {
    pub program_id: Pubkey,
    pub accounts: Vec<u16>,
    pub data: Vec<u8>,
}
