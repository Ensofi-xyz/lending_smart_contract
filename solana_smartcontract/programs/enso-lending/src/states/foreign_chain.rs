use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct ForeignChain {
    /// Emitter chain. Cannot equal `1` (Solana's Chain ID).
    pub chain_id: u16,
    #[max_len(100)]
    pub chain_address: String,
     #[max_len(100)]
    pub emitter_address: String,
    pub bump: u8,
}