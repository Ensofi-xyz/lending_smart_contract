use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::{common::constant::{ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED, OPERATE_SYSTEM_PUBKEY}, LoanOfferAccount, LoanOfferError, LoanOfferStatus, SystemRevertEvent};

#[derive(Accounts)]
#[instruction(offer_id: String)]
pub struct SystemRevertStatus<'info> {
  #[account(mut)]
  pub system: Signer<'info>,
  /// CHECK: This is the account used to make a seeds
  pub borrower: AccountInfo<'info>,
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
}

impl<'info> SystemRevertStatus<'info> {
  pub fn system_revert_status(&mut self) -> Result<()> {
    if self.system.key() != Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() {
      return err!(LoanOfferError::InvalidSystem);
    }

    let loan_offer = &mut self.loan_offer;
    if loan_offer.status != LoanOfferStatus::Liquidating {
      return err!(LoanOfferError::InvalidOfferStatus);
    }

    loan_offer.liquidating_price = None;
    loan_offer.liquidating_at = None;
    loan_offer.status = LoanOfferStatus::FundTransferred;

    self.emit_event_revert_status()?;

    Ok(())
  }

  fn emit_event_revert_status(&self) -> Result<()> {
    emit!(SystemRevertEvent {
      offer_id: self.loan_offer.offer_id.clone(),
      status: self.loan_offer.status.clone(),
    });

    Ok(())
  }
}