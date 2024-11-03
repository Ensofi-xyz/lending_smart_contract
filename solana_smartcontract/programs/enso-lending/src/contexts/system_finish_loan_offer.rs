use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};
use crate::{
  common::{
    constant::LoanOfferStatus, RepayOfferError
  }, duration_to_year, states::loan_offer::LoanOfferAccount, SystemFinishLoanOfferEvent, ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED
};

#[derive(Accounts)]
#[instruction(loan_offer_id: String)]
pub struct SystemFinishLoanOffer<'info> {
  #[account(mut)]
  pub system: Signer<'info>,
  #[account(
    mut,
    associated_token::mint = mint_asset,
    associated_token::authority = system
  )]
  pub system_ata: Account<'info, TokenAccount>,
  #[account(
    constraint = mint_asset.key() == loan_offer.lend_mint_token @ RepayOfferError::InvalidMintAsset,
  )]
  pub mint_asset: Account<'info, Mint>,
  #[account(
    constraint = lender.key() == loan_offer.lender @ RepayOfferError::InvalidLender
  )]
  pub lender: SystemAccount<'info>,
  #[account(
    mut,
    associated_token::mint = mint_asset,
    associated_token::authority = lender
  )]
  pub lender_ata_asset: Account<'info, TokenAccount>,
  #[account(mut)]
  pub borrower: SystemAccount<'info>,
  #[account(
    mut,
    constraint = loan_offer.status == LoanOfferStatus::BorrowerPaid 
    || loan_offer.status == LoanOfferStatus::Liquidated @ RepayOfferError::InvalidOfferStatus,
    seeds = [
      ENSO_SEED.as_ref(),
      LOAN_OFFER_ACCOUNT_SEED.as_ref(),
      borrower.key().as_ref(),
      loan_offer_id.as_bytes(),
      crate::ID.key().as_ref()
    ],
    bump = loan_offer.bump
  )]
  pub loan_offer: Account<'info, LoanOfferAccount>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}

impl<'info> SystemFinishLoanOffer<'info> {
  pub fn system_finish_loan_offer(&mut self, loan_amount: u64, waiting_interest: u64) -> Result<()>  {
    let total_repay_to_lender = self.get_total_repay(loan_amount, waiting_interest);

    // TODO: Allow system finish contract at any time
    // let current_timestamp = Clock::get().unwrap().unix_timestamp;
    // let end_borrowed_loan_offer = self.loan_offer.started_at + self.loan_offer.duration as i64;

    // if current_timestamp < end_borrowed_loan_offer {
    //   return err!(RepayOfferError::TimeUnmetException);
    // }

    self.transfer_asset_to_lender(loan_amount, total_repay_to_lender)?;
    self.loan_offer.status = LoanOfferStatus::Finished;

    self.emit_event_system_finish_loan_offer(
      loan_amount
    )?;

    Ok(())
  }

  fn transfer_asset_to_lender(&mut self, loan_amount: u64, total_repay_to_lender: u64) -> Result<()> {
    if loan_amount != self.loan_offer.borrow_amount {
      return err!(RepayOfferError::InvalidLendAmount);
    }

    if total_repay_to_lender > self.system_ata.amount {
      return err!(RepayOfferError::NotEnoughAmount);
    }

    self.process_transfer_lend_asset(total_repay_to_lender)?;

    Ok(())
  }

  fn process_transfer_lend_asset(&mut self, total_repay: u64) -> Result<()> {
    let ctx = CpiContext::new(
      self.token_program.to_account_info(),  
      TransferChecked {
        from: self.system_ata.to_account_info(),
        mint: self.mint_asset.to_account_info(),
        to: self.lender_ata_asset.to_account_info(),
        authority: self.system.to_account_info(),
    });

    transfer_checked(
      ctx,
      total_repay,
      self.mint_asset.decimals,
    )
  }

  fn emit_event_system_finish_loan_offer(
    &mut self,
    loan_amount: u64
  ) -> Result<()> {
    emit!(SystemFinishLoanOfferEvent {
      system: self.system.key(),
      lender: self.lender.key(),
      borrower: self.borrower.key(),
      interest: self.loan_offer.interest,
      loan_amount,
      loan_offer_id: self.loan_offer.offer_id.clone(),
      tier_id: self.loan_offer.tier_id.clone(),
      status: self.loan_offer.status,
    });

    Ok(())
  }

  fn get_total_repay(&self, loan_amount: u64, waiting_interest: u64) -> u64 {
    let loan_interest_percent = self.loan_offer.interest / 100.0;

    let lender_fee_percent = self.loan_offer.lender_fee_percent / 100.0;

    let time_borrowed = duration_to_year(self.loan_offer.duration);

    let interest_loan_amount = (loan_amount as f64) * loan_interest_percent * time_borrowed;
    let lender_fee_amount = lender_fee_percent * (interest_loan_amount as f64);

    return (loan_amount as f64 + interest_loan_amount + waiting_interest as f64 - lender_fee_amount) as u64;
  }
}