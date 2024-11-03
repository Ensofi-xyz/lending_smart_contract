use std::ops::Add;

use anchor_lang::{prelude::*, solana_program::{program::invoke_signed, system_instruction}};

use crate::{
  common::{ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED}, DepositCollateralLoanOfferEvent, LoanOfferAccount, LoanOfferError, LoanOfferStatus
};

#[derive(Accounts)]
#[instruction(
  offer_id: String, 
  tier_id: String,
  amount: u64
)]
pub struct DepositCollateralLoanOfferNative<'info> {
  #[account(mut)]
  pub borrower: Signer<'info>,
  #[account(
    mut,
    constraint = 
      loan_offer.status == LoanOfferStatus::Matched || loan_offer.status == LoanOfferStatus::FundTransferred 
      @ LoanOfferError::CanNotDepositCollateralToContractThatNotAvailable,
    seeds = [
      ENSO_SEED.as_ref(),
      LOAN_OFFER_ACCOUNT_SEED.as_ref(),
      borrower.key().as_ref(),
      offer_id.as_bytes(),
      crate::ID.key().as_ref()
    ],
    bump = loan_offer.bump
  )]
  pub loan_offer: Account<'info, LoanOfferAccount>,
  pub system_program: Program<'info, System>,
}

impl<'info> DepositCollateralLoanOfferNative<'info> {
  pub fn deposit_collateral_loan_offer_native(&mut self, amount: u64) -> Result<()> {
    self.deposit_collateral(amount)?;

    let before_collateral_amount = self.loan_offer.collateral_amount;
    self.loan_offer.collateral_amount = before_collateral_amount.add(amount);

    self.emit_event_deposit_collateral_loan_offer()?;
    
    Ok(())
  }

  fn emit_event_deposit_collateral_loan_offer(&self) -> Result<()> {
    emit!(DepositCollateralLoanOfferEvent {
      tier_id: self.loan_offer.tier_id.clone(),
      lend_offer_id: self.loan_offer.lend_offer_id.clone(),
      interest: self.loan_offer.interest,
      borrow_amount: self.loan_offer.borrow_amount,
      lender_fee_percent: self.loan_offer.lender_fee_percent,
      duration: self.loan_offer.duration,
      lend_mint_token: self.loan_offer.lend_mint_token,
      lender: self.loan_offer.lender,
      offer_id: self.loan_offer.offer_id.clone(),
      borrower: self.loan_offer.borrower,
      collateral_mint_token: self.loan_offer.collateral_mint_token,
      collateral_amount: self.loan_offer.collateral_amount,
      status: self.loan_offer.status,
      borrower_fee_percent: self.loan_offer.borrower_fee_percent,
      started_at: self.loan_offer.started_at,
    });
    
    Ok(())
  }

  fn deposit_collateral(&self, amount: u64) -> Result<()> {
     let transfer_instruction = system_instruction::transfer(
      &self.borrower.key(), 
      &self.loan_offer.key(), 
      amount
    );
    
     invoke_signed(
       &transfer_instruction,
       &[
         self.borrower.to_account_info(),
         self.loan_offer.to_account_info(),          
         self.system_program.to_account_info()
       ],
       &[],  
     )?;
 
     Ok(())
  }
}