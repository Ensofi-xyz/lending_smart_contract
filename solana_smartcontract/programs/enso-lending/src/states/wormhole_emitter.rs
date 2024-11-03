use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct WormholeEmitter {
    pub chain: u16,
    #[max_len(100)]
    pub address: String,
    pub bump: u8,
}