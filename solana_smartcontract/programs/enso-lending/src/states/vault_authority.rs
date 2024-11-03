use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct VaultAuthority {
    pub initializer: Pubkey,
    pub bump: u8,
}
