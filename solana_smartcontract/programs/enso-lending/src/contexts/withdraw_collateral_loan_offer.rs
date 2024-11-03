use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{
  common::{
    ASSET_SEED, ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED
  }, health_ratio::{self, HealthRatioParams}, states::{
    Asset, LoanOfferAccount
  }, LoanOfferError, LoanOfferStatus, VaultAuthority, WithdrawCollateralEvent, VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED
};

#[derive(Accounts)]
#[instruction(loan_offer_id: String)]
pub struct WithdrawCollateralLoanOffer<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    #[account(
      constraint = collateral_mint_asset.key() == collateral_asset.token_mint @ LoanOfferError::InvalidCollateralMintAsset,
    )]
    pub collateral_mint_asset: Account<'info, Mint>,
    #[account(
      constraint = lend_mint_asset.key() == lend_asset.token_mint @ LoanOfferError::InvalidLendMintAsset,
    )]
    pub lend_mint_asset: Account<'info, Mint>,
    #[account(
      constraint = lend_asset.is_lend == true @ LoanOfferError::InvalidAssetAccount,
      seeds = [
        ENSO_SEED.as_ref(),
        ASSET_SEED.as_ref(),
        lend_mint_asset.key().as_ref(),
        crate::ID.key().as_ref()
      ],
      bump = lend_asset.bump
    )]
    pub lend_asset: Account<'info, Asset>,
    #[account(
      constraint = collateral_asset.is_collateral == true @ LoanOfferError::InvalidAssetAccount,
      seeds = [
        ENSO_SEED.as_ref(),
        ASSET_SEED.as_ref(),
        collateral_mint_asset.key().as_ref(),
        crate::ID.key().as_ref()
      ],
      bump = collateral_asset.bump
    )]
    pub collateral_asset: Account<'info, Asset>,
    #[account(
      mut,
      associated_token::mint = collateral_mint_asset,
      associated_token::authority = borrower
    )]
    pub borrower_ata_collateral_asset: Box<Account<'info, TokenAccount>>,
    #[account(
      mut,
      constraint = loan_offer.status == LoanOfferStatus::FundTransferred @ LoanOfferError::NotAvailableToWithdraw,
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
      constraint = lend_price_feed_account.key() == lend_asset.price_feed_account @ LoanOfferError::InvalidPriceFeedAccount,
    )]
    pub lend_price_feed_account: Account<'info, PriceUpdateV2>,
    #[account(
      constraint = collateral_price_feed_account.key() == collateral_asset.price_feed_account @ LoanOfferError::InvalidPriceFeedAccount,
    )]
    pub collateral_price_feed_account: Account<'info, PriceUpdateV2>,
    pub token_program: Program<'info, Token>,
}

impl<'info> WithdrawCollateralLoanOffer<'info> {
  pub fn withdraw_collateral_loan_offer(&mut self, loan_offer_id: String, withdraw_amount: u64) -> Result<()> { 
    self.validate_withdraw_collateral(withdraw_amount)?;

    self.transfer_collateral_to_borrower(withdraw_amount)?;

    self.loan_offer.collateral_amount = self.loan_offer.collateral_amount - withdraw_amount;

    self.emit_event_withdraw_collateral(
      loan_offer_id,
      withdraw_amount,
    )?;

    Ok(())
  }

  fn validate_withdraw_collateral(&self, withdraw_amount: u64) -> Result<()> {
    if withdraw_amount > self.loan_offer.collateral_amount {
      return err!(LoanOfferError::NotEnoughCollateral);
    }

    let remaining_collateral = self.loan_offer.collateral_amount - withdraw_amount;

    health_ratio::validate_health_ratio(HealthRatioParams {
      collateral_price_feed_account: &self.collateral_price_feed_account,
      collateral_amount: remaining_collateral,
      collateral_price_feed_id: self.collateral_asset.price_feed_id.clone(),
      collateral_max_price_age_seconds: self.collateral_asset.max_price_age_seconds,
      collateral_decimals: self.collateral_asset.decimals,
      lend_price_feed_account: &self.lend_price_feed_account,
      lend_amount: self.loan_offer.borrow_amount,
      lend_price_feed_id: self.lend_asset.price_feed_id.clone(),
      lend_max_price_age_seconds: self.lend_asset.max_price_age_seconds,
      lend_decimals: self.lend_asset.decimals,
    })?;

    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    let end_borrowed_loan_offer = self.loan_offer.started_at + self.loan_offer.duration as i64;

    if current_timestamp > end_borrowed_loan_offer {
      return err!(LoanOfferError::LoanOfferExpired)?;
    }

    Ok(())
  }

  fn transfer_collateral_to_borrower(& self, collateral_amount: u64) -> Result<()> {
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
      collateral_amount,
      self.collateral_mint_asset.decimals,
    )
  }

  fn emit_event_withdraw_collateral(&mut self, loan_offer_id: String, withdraw_amount: u64) -> Result<()> {
    emit!(WithdrawCollateralEvent {
      borrower: self.borrower.key(),
      loan_offer_id,
      collateral_amount: self.loan_offer.collateral_amount,
      withdraw_amount,
    });

    Ok(())
  }
}