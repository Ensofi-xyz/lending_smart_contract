use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct ForeignEmitter {
    /// Emitter chain. Cannot equal `1` (Solana's Chain ID).
    pub chain: u16,
    #[max_len(100)]
    pub address: String,
    pub bump: u8,
}