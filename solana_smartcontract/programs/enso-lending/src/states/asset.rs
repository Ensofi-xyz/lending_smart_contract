use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct Asset {
  pub token_mint: Pubkey,
  pub max_price_age_seconds: u64,
  pub decimals: u8,
  pub is_collateral: bool,
  pub is_lend: bool,
  #[max_len(100)]
  pub price_feed_id: String,
  pub price_feed_account: Pubkey,
  pub bump: u8,
  #[max_len(30)]
  pub name: String,
  #[max_len(100)]
  pub token_address: Option<String>,
  pub chain_id: u16,
}