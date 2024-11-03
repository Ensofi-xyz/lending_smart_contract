use anchor_lang::prelude::*;

use crate::{common::{ENSO_SEED, LEND_OFFER_ACCOUNT_SEED}, EditLendOfferEvent, LendOfferAccount, LendOfferError, LendOfferStatus, MAX_ALLOWED_INTEREST};

#[derive(Accounts)]
#[instruction(offer_id: String)]
pub struct EditLendOffer<'info> {
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

impl<'info> EditLendOffer<'info> {
    pub fn edit_lend_offer(&mut self, interest: f64) -> Result<()> {
      if interest <= (0 as f64) {
        return err!(LendOfferError::InterestGreaterThanZero);
      }

      if interest >= MAX_ALLOWED_INTEREST {
        return err!(LendOfferError::InterestOverLimit);
      }

      let lend_offer = &mut self.lend_offer;
      lend_offer.interest = interest;

      self.emit_event_edit_lend_offer()?;

      Ok(())
    }

    fn emit_event_edit_lend_offer(&mut self) -> Result<()> {
      emit!(EditLendOfferEvent {
        lender: self.lender.key(),
        interest: self.lend_offer.interest,
        lender_fee_percent: self.lend_offer.lender_fee_percent,
        amount: self.lend_offer.amount,
        duration: self.lend_offer.duration,
        offer_id: self.lend_offer.offer_id.clone(),
    });
    
    Ok(())
    }
}