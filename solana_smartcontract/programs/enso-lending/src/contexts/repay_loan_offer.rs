use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{
  amount::TotalRepayLoanAmountParams, common::{
    ENSO_SEED,
    LOAN_OFFER_ACCOUNT_SEED, 
    VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED 
  }, utils, LoanOfferAccount, LoanOfferError, LoanOfferStatus, RepayOfferError, SystemRepayLoanOfferEvent, VaultAuthority, HOT_WALLET_PUBKEY
};

#[derive(Accounts)]
#[instruction(loan_offer_id: String)]
pub struct RepayLoanOffer<'info> {
  #[account(mut)]
  pub borrower: Signer<'info>,
  #[account(
    constraint = collateral_mint_asset.key() == loan_offer.collateral_mint_token @ LoanOfferError::InvalidCollateralMintAsset,
  )]
  pub collateral_mint_asset: Box<Account<'info, Mint>>,
  #[account(
    constraint = lend_mint_asset.key() == loan_offer.lend_mint_token @ LoanOfferError::InvalidLendMintAsset,
  )]
  pub lend_mint_asset: Box<Account<'info, Mint>>,
  #[account(
    mut,
    associated_token::mint = lend_mint_asset,
    associated_token::authority = borrower
  )]
  pub borrower_ata_lend_asset: Box<Account<'info, TokenAccount>>,
  #[account(
    mut,
    associated_token::mint = collateral_mint_asset,
    associated_token::authority = borrower
  )]
  pub borrower_ata_collateral_asset: Box<Account<'info, TokenAccount>>,
  #[account(
    mut,
    constraint = loan_offer.status == LoanOfferStatus::FundTransferred @ RepayOfferError::LoanOfferIsNotAvailable,
    seeds = [
      ENSO_SEED.as_ref(),
      LOAN_OFFER_ACCOUNT_SEED.as_ref(),
      borrower.key().as_ref(),
      loan_offer_id.as_bytes(),
      crate::ID.key().as_ref()
    ],
    bump
  )]
  pub loan_offer: Box<Account<'info, LoanOfferAccount>>,
  #[account(
    mut,
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
  #[account(
    mut,
    associated_token::mint = lend_mint_asset,
    associated_token::authority = Pubkey::from_str(HOT_WALLET_PUBKEY).unwrap()
  )]
  pub hot_wallet_ata_lend_asset: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
}

impl<'info> RepayLoanOffer<'info> {
  pub fn repay_loan_offer(&mut self) -> Result<()> {
    self.validate_loan_offer()?;
    
    let total_amount = utils::amount::get_total_repay_loan_amount(TotalRepayLoanAmountParams {
      borrow_amount: self.loan_offer.borrow_amount,
      borrower_fee_percent: self.loan_offer.borrower_fee_percent,
      duration: self.loan_offer.duration,
      interest: self.loan_offer.interest,
  });
    
    if total_amount > self.borrower_ata_lend_asset.amount {
      return err!(RepayOfferError::NotEnoughAmount);
    } 

    self.repay_lend_asset_to_hot_wallet(total_amount)?;
    self.transfer_collateral_to_borrower()?;

    self.loan_offer.status = LoanOfferStatus::BorrowerPaid;

    self.emit_event_repay_loan_offer()?;
    Ok(())
  }

  fn validate_loan_offer(&self) -> Result<()> {
    // No need to check timestamp (already check status)
    // let current_timestamp = Clock::get().unwrap().unix_timestamp;
    // let end_borrowed_loan_offer = self.loan_offer.started_at + self.loan_offer.duration as i64;

    // if current_timestamp > end_borrowed_loan_offer {
    //   return err!(LoanOfferError::LoanOfferExpired);
    // }

    Ok(())
  }    

  fn repay_lend_asset_to_hot_wallet(&self, repay_amount: u64) -> Result<()> {
    let cpi_ctx = CpiContext::new(
      self.token_program.to_account_info(), 
      TransferChecked {
        from: self.borrower_ata_lend_asset.to_account_info(),
        mint: self.lend_mint_asset.to_account_info(),
        to: self.hot_wallet_ata_lend_asset.to_account_info(),
        authority: self.borrower.to_account_info(),
      }
    );

    transfer_checked(
      cpi_ctx,
      repay_amount,
      self.lend_mint_asset.decimals,
    )
  }

  fn transfer_collateral_to_borrower(& self) -> Result<()> {
    let borrower_pub_key = self.borrower.key();
    let program_id = crate::ID.key();

    let signer: &[&[&[u8]]] = &[&[ 
      ENSO_SEED.as_ref(), 
      borrower_pub_key.as_ref(), 
      VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED.as_ref(), 
      program_id.as_ref(), 
      &[self.vault_authority.bump] 
    ]];

    let cpi_ctx = CpiContext::new_with_signer(
      self.token_program.to_account_info(), 
      TransferChecked {
        from: self.vault.to_account_info(),
        mint: self.collateral_mint_asset.to_account_info(),
        to: self.borrower_ata_collateral_asset.to_account_info(),
        authority: self.vault_authority.to_account_info(),
      },
      signer
    );

    transfer_checked(
      cpi_ctx,
      self.loan_offer.collateral_amount,
      self.collateral_mint_asset.decimals,
    )
  }

  fn emit_event_repay_loan_offer(&mut self) -> Result<()> {
    emit!(SystemRepayLoanOfferEvent {
      lender: self.loan_offer.lender.key(),
      borrower: self.borrower.key(),
      interest: self.loan_offer.interest,
      loan_amount: self.loan_offer.borrow_amount,
      loan_offer_id: self.loan_offer.offer_id.clone(),
      tier_id: self.loan_offer.tier_id.clone(),
      collateral_amount: self.loan_offer.collateral_amount,
      status: self.loan_offer.status,
    });
          
    Ok(())
  }
}