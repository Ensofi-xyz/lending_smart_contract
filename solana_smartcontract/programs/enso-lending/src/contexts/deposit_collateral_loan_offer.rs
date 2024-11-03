use std::ops::Add;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, TransferChecked, transfer_checked};

use crate::{
  common::{
    ENSO_SEED, 
    LOAN_OFFER_ACCOUNT_SEED,
    VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED
  }, 
  DepositCollateralLoanOfferEvent, 
  LoanOfferAccount, 
  LoanOfferError, 
  LoanOfferStatus, VaultAuthority, 
};

#[derive(Accounts)]
#[instruction(
  offer_id: String, 
  tier_id: String,
  amount: u64
)]
pub struct DepositCollateralLoanOffer<'info> {
  #[account(mut)]
  pub borrower: Signer<'info>,
  #[account(
    constraint = collateral_mint_asset.key() == loan_offer.collateral_mint_token @ LoanOfferError::InvalidCollateralMintAsset,
  )]
  pub collateral_mint_asset: Account<'info, Mint>,
  #[account(
    mut,
    constraint = borrower_ata_asset.amount >= amount @ LoanOfferError::NotEnoughAmount,
    associated_token::mint = collateral_mint_asset,
    associated_token::authority = borrower
  )]
  pub borrower_ata_asset: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = loan_offer.status == LoanOfferStatus::Matched || loan_offer.status == LoanOfferStatus::FundTransferred 
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
  #[account(
    constraint = vault_authority.initializer.key() == borrower.key() @ LoanOfferError::InvalidInitializerVaultAuthority,
    seeds = [
      ENSO_SEED.as_ref(), 
      borrower.key().as_ref(),
      VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED.as_ref(), 
      crate::ID.key().as_ref(), 
    ],
    bump = vault_authority.bump
  )]
  pub vault_authority: Box<Account<'info, VaultAuthority>>,
  #[account(
    mut,
    associated_token::mint = collateral_mint_asset,
    associated_token::authority = vault_authority
  )]
  pub vault: Box<Account<'info, TokenAccount>>,
  pub token_program: Program<'info, Token>,
}

impl<'info> DepositCollateralLoanOffer<'info> {
  pub fn deposit_collateral_loan_offer(&mut self, amount: u64) -> Result<()> {
    self.deposit_collateral(amount)?;

    let before_collateral_amount = self.loan_offer.collateral_amount;
    self.loan_offer.collateral_amount = before_collateral_amount.add(amount);

    self.emit_event_deposit_collateral_loan_offer()?;
    Ok(())
  }

  fn deposit_collateral(&self, collateral_amount: u64) -> Result<()> {
    let cpi_context = CpiContext::new(
      self.token_program.to_account_info(), 
      TransferChecked {
        from: self.borrower_ata_asset.to_account_info(),
        mint: self.collateral_mint_asset.to_account_info(),
        to: self.vault.to_account_info(),
        authority: self.borrower.to_account_info(),
    }
    );

    transfer_checked(
      cpi_context,
      collateral_amount,
      self.collateral_mint_asset.decimals,
    )
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
}