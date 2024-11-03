use anchor_lang::prelude::*;
use crate::{
  ENSO_SEED, LEND_OFFER_ACCOUNT_SEED,
  common::{
    LendOfferCancelRequestEvent, 
    LendOfferStatus, 
    LendOfferError
  },
  states::lend_offer::LendOfferAccount
};

#[derive(Accounts)]
#[instruction(offer_id: String)]
pub struct CancelLendOffer<'info> {
#[account(mut)]
  pub lender: Signer<'info>,
  #[account(
    mut,
    constraint = lend_offer.status == LendOfferStatus::Created @ LendOfferError::InvalidOfferStatus,
    seeds = [
      ENSO_SEED.as_ref(),
      LEND_OFFER_ACCOUNT_SEED.as_ref(),  
      lender.key().as_ref(), 
      offer_id.as_bytes(),
      crate::ID.key().as_ref(), 
    ],
    bump = lend_offer.bump
  )]
  pub lend_offer: Account<'info, LendOfferAccount>,
}

impl<'info> CancelLendOffer<'info> {
  pub fn cancel_lend_offer(&mut self) -> Result<()>  {
    self.lend_offer.status = LendOfferStatus::Canceling;

    self.emit_event_cancel_lend_offer()?;

    Ok(())
  }

  fn emit_event_cancel_lend_offer(&mut self) -> Result<()> {
    emit!(LendOfferCancelRequestEvent {
      lender: self.lender.key(),
      amount: self.lend_offer.amount,
      duration: self.lend_offer.duration,
      interest: self.lend_offer.interest,
      lender_fee_percent: self.lend_offer.lender_fee_percent,
      offer_id: self.lend_offer.offer_id.clone()
    });


    Ok(())
  }
}