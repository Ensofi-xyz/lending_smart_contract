use anchor_lang::{
    event,
    prelude::{borsh, AnchorDeserialize, AnchorSerialize, Pubkey},
};

use crate::LoanOfferStatus;

#[event]
pub struct InitSettingAccountEvent {
    pub amount: u64,
    pub duration: u64,
    pub owner: Pubkey,
    pub receiver: Pubkey,
    pub tier_id: String,
    pub lender_fee_percent: f64,
    pub borrower_fee_percent: f64
}

#[event]
pub struct EditSettingAccountEvent {
    pub receiver: Pubkey,
    pub tier_id: String,
    pub amount: u64,
    pub duration: u64,
    pub lender_fee_percent: f64,
}

#[event]
pub struct CloseSettingAccountEvent {
    pub tier_id: String,
}

#[event]
pub struct InitAssetEvent {
    pub token_mint: Pubkey,
    pub max_price_age_seconds: u64,
    pub decimals: u8,
    pub is_collateral: bool,
    pub is_lend: bool,
    pub price_feed_id: String,
    pub price_feed_account: Pubkey,
    pub bump: u8,
    pub name: String,
    pub token_address: Option<String>,
    pub chain_id: u16,
}

#[event]
pub struct EditAssetEvent {
    pub token_mint: Pubkey,
    pub max_price_age_seconds: u64,
    pub decimals: u8,
    pub is_collateral: bool,
    pub is_lend: bool,
    pub price_feed_id: String,
    pub price_feed_account: Pubkey,
    pub bump: u8,
    pub name: String
}

#[event]
pub struct CreateLendOfferEvent {
    pub lender: Pubkey,
    pub interest: f64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub amount: u64,
    pub offer_id: String,
    pub tier_id: String,
}

#[event]
pub struct EditLendOfferEvent {
    pub lender: Pubkey,
    pub interest: f64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub amount: u64,
    pub offer_id: String,
}

#[event]
pub struct LendOfferCancelRequestEvent {
    pub lender: Pubkey,
    pub interest: f64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub amount: u64,
    pub offer_id: String,
}

#[event]
pub struct LendOfferCanceledEvent {
    pub lender: Pubkey,
    pub interest: f64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub amount: u64,
    pub offer_id: String,
}

#[event]
pub struct LoanOfferCreateRequestEvent {
    pub tier_id: String,
    pub lend_offer_id: String,
    pub interest: f64,
    pub borrow_amount: u64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub lend_mint_token: Pubkey,
    pub lender: Pubkey,
    pub offer_id: String,
    pub borrower: Pubkey,
    pub collateral_mint_token: Pubkey,
    pub collateral_amount: u64,
    pub status: LoanOfferStatus,
    pub borrower_fee_percent: f64,
    pub started_at: i64,
}

#[event]
pub struct LoanOfferUpdateEvent {
    pub tier_id: String,
    pub lend_offer_id: String,
    pub interest: f64,
    pub borrow_amount: u64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub lend_mint_token: Pubkey,
    pub lender: Pubkey,
    pub offer_id: String,
    pub borrower: Pubkey,
    pub collateral_mint_token: Pubkey,
    pub collateral_amount: u64,
    pub status: LoanOfferStatus,
    pub borrower_fee_percent: f64,
    pub started_at: i64,
}

#[event]
pub struct WithdrawCollateralEvent {
    pub borrower: Pubkey,
    pub withdraw_amount: u64,
    pub loan_offer_id: String,
    pub collateral_amount: u64,
}

#[event]
pub struct UnhealthyRatioDetectedEvent  {
    pub borrower: Pubkey,
    pub withdraw_amount: u64,
    pub loan_offer_id: String,
    pub collateral_amount: u64,
}

#[event]
pub struct LoanOfferExpiredEvent  {
    pub borrower: Pubkey,
    pub withdraw_amount: u64,
    pub loan_offer_id: String,
    pub collateral_amount: u64,
}

#[event]
pub struct DepositCollateralLoanOfferEvent {
    pub tier_id: String,
    pub lend_offer_id: String,
    pub interest: f64,
    pub borrow_amount: u64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub lend_mint_token: Pubkey,
    pub lender: Pubkey,
    pub offer_id: String,
    pub borrower: Pubkey,
    pub collateral_mint_token: Pubkey,
    pub collateral_amount: u64,
    pub status: LoanOfferStatus,
    pub borrower_fee_percent: f64,
    pub started_at: i64,
}

#[event]
pub struct RepayLoanOfferEvent {
  pub borrower: Pubkey,
  pub loan_offer_id: String,
  pub repay_amount: u64,
  pub borrower_fee_percent: f64,
  pub status: LoanOfferStatus,
}

#[event]
pub struct LiquidatingCollateralEvent {
    pub offer_id: String,
    pub liquidating_price: Option<f64>,
    pub liquidating_at: Option<i64>,
}

#[event]
pub struct LiquidatedCollateralEvent {
    pub loan_offer_id: String,
    pub liquidated_price: u64,
    pub liquidated_tx: String,
    pub system: Pubkey,
    pub lender: Pubkey,
    pub borrower: Pubkey,
    pub remaining_fund_to_borrower: u64,
    pub collateral_swapped_amount: u64,
    pub status: LoanOfferStatus,
}

#[event]
pub struct SystemRepayLoanOfferEvent {
    pub lender: Pubkey,
    pub borrower: Pubkey,
    pub interest: f64,
    pub loan_amount: u64,
    pub collateral_amount: u64,
    pub loan_offer_id: String,
    pub tier_id: String,
    pub status: LoanOfferStatus,
}

#[event]
pub struct SystemFinishLoanOfferEvent {
    pub system: Pubkey,
    pub lender: Pubkey,
    pub borrower: Pubkey,
    pub interest: f64,
    pub loan_amount: u64,
    pub loan_offer_id: String,
    pub tier_id: String,
    pub status: LoanOfferStatus,
}

#[event]
pub struct SystemRevertEvent {
    pub offer_id: String,
    pub status: LoanOfferStatus,
}

#[event]
pub struct InitForeignEmitterEvent {
    pub chain: u16,
    pub address: String,
}

#[event]
pub struct CancelLoanOfferCrossChainEvent {
    pub borrower: Pubkey,
    pub lend_offer_id: String,
    pub loan_offer_id: String,
    pub target_chain: u16,
    pub target_address: String,
}