pub use anchor_lang::prelude::*;

use crate::LendOfferStatus;

#[account]
#[derive(InitSpace, Debug)]
pub struct LendOfferAccount {
  pub interest: f64,
  pub lender_fee_percent: f64,
  pub duration: u64,
  #[max_len(50)]
  pub offer_id: String,
  pub lender: Pubkey,
  pub lend_mint_token: Pubkey,
  pub amount: u64,
  pub bump: u8,
  pub status: LendOfferStatus
}
