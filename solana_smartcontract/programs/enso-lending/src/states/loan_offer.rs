pub use anchor_lang::prelude::*;

use crate::LoanOfferStatus;

#[account]
#[derive(InitSpace, Debug)]
pub struct LoanOfferAccount {
  #[max_len(50)]
  pub tier_id: String,
  #[max_len(50)]
  pub lend_offer_id: String,
  pub interest: f64,
  pub borrow_amount: u64,
  pub lender_fee_percent: f64,
  pub duration: u64,
  pub lend_mint_token: Pubkey,
  pub lender: Pubkey,
  #[max_len(50)]
  pub offer_id: String,
  pub borrower: Pubkey,
  pub collateral_mint_token: Pubkey,
  pub collateral_amount: u64,
  pub request_withdraw_amount: Option<u64>,
  pub status: LoanOfferStatus,
  pub borrower_fee_percent: f64,
  pub started_at: i64,
  pub liquidating_at: Option<i64>,
  pub liquidating_price: Option<f64>,
  #[max_len(50)]
  pub liquidated_tx: Option<String>,
  pub liquidated_price: Option<u64>,
  pub bump: u8,
}
