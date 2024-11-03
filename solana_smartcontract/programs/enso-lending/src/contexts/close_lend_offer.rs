use std::str::FromStr;

use anchor_lang::prelude::*;
use crate::{
  common::{
    constant::{LendOfferStatus, OPERATE_SYSTEM_PUBKEY},
    error::LendOfferError
  }, 
  states::lend_offer::LendOfferAccount
};

#[derive(Accounts)]
pub struct CloseLendOffer<'info> {
  #[account(
    mut,
    constraint = signer.key() == lend_offer.lender || signer.key() == Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() @ LendOfferError::InvalidSigner
  )]
  pub signer: Signer<'info>,
  #[account(
    constraint = lender.key() == lend_offer.lender @ LendOfferError::InvalidLender
  )]
  ///CHECK: This account is used to check the validate of lend offer account
  pub lender: AccountInfo<'info>,
  #[account(
    mut,
    close = lender
  )]
  pub lend_offer: Account<'info, LendOfferAccount>,
  pub system_program: Program<'info, System>,
}

impl<'info> CloseLendOffer<'info> {
  pub fn validate_lend_offer(&mut self, offer_id: String) -> Result<()> {
    if self.lend_offer.status != LendOfferStatus::Canceled {
      return err!(LendOfferError::InvalidOfferStatus);
    }

    if self.lend_offer.offer_id != offer_id {
      return err!(LendOfferError::InvalidOfferId);
    }

    Ok(())
  }
}