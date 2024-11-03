use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::{
  common::constant::{
    ENSO_SEED, 
    LOAN_OFFER_ACCOUNT_SEED, 
    OPERATE_SYSTEM_PUBKEY
  }, 
  LiquidatingCollateralEvent, 
  LoanOfferAccount, 
  LoanOfferError, 
  LoanOfferStatus, 
  HOT_WALLET_PUBKEY
};

#[derive(Accounts)]
#[instruction(offer_id: String)]
pub struct StartLiquidateLoanOfferNativeExpired<'info> {
  #[account(
    mut,
    constraint = system.key() == Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() @ LoanOfferError::InvalidSystem
  )]
  pub system: Signer<'info>,
  pub borrower: SystemAccount<'info>,
  #[account(
    mut,
    seeds = [
      ENSO_SEED.as_ref(),
      LOAN_OFFER_ACCOUNT_SEED.as_ref(),
      borrower.key().as_ref(),
      offer_id.as_bytes(),
      crate::ID.key().as_ref()
    ],
    bump
  )]
  pub loan_offer: Account<'info, LoanOfferAccount>,
  #[account(
    mut,
    constraint = hot_wallet.key() == Pubkey::from_str(HOT_WALLET_PUBKEY).unwrap() @ LoanOfferError::InvalidHotWallet
  )]
  pub hot_wallet: SystemAccount<'info>,
}

impl<'info> StartLiquidateLoanOfferNativeExpired<'info> {
  pub fn start_liquidate_loan_offer_native_expired(&mut self) -> Result<()> {
    self.validate_expired_loan_offer()?;
    
    let loan_offer = &mut self.loan_offer;
    if loan_offer.status != LoanOfferStatus::FundTransferred {
      return err!(LoanOfferError::InvalidOfferStatus);
    }

    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    loan_offer.liquidating_at = Some(current_timestamp);
    loan_offer.status = LoanOfferStatus::Liquidating;

    self.loan_offer.sub_lamports(self.loan_offer.collateral_amount)?;
    self.hot_wallet.add_lamports(self.loan_offer.collateral_amount)?;

    self.emit_event_start_liquidate_contract()?;

    Ok(())
  }

  fn emit_event_start_liquidate_contract(&self) -> Result<()> {
    emit!(LiquidatingCollateralEvent {
      offer_id: self.loan_offer.offer_id.clone(),
      liquidating_at: self.loan_offer.liquidating_at,
      liquidating_price: self.loan_offer.liquidating_price,
    });

    Ok(())
  }

  fn validate_expired_loan_offer(&self) -> Result<()> {
    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    let end_borrowed_loan_offer = self.loan_offer.started_at + self.loan_offer.duration as i64;

    if current_timestamp < end_borrowed_loan_offer {
      return err!(LoanOfferError::LoanOfferNotExpired);
    }

    Ok(())
  }

}