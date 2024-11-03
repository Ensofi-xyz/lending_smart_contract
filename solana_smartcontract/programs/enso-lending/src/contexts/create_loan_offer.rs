use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{
    common::{
      ASSET_SEED, ENSO_SEED, LEND_OFFER_ACCOUNT_SEED, LOAN_OFFER_ACCOUNT_SEED, SETTING_ACCOUNT_SEED
    }, health_ratio::{self, HealthRatioParams}, Asset, LendOfferAccount, LendOfferStatus, LoanOfferAccount, LoanOfferCreateRequestEvent, LoanOfferError, LoanOfferStatus, SettingAccount, VaultAuthority, DISCRIMINATOR, VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED
};

#[derive(Accounts)]
#[instruction(
  offer_id: String, 
  lend_offer_id: String, 
  tier_id: String, 
  collateral_amount: u64
)]
pub struct CreateLoanOffer<'info> {
  #[account(mut)]
  pub borrower: Signer<'info>,
  #[account(
    constraint = collateral_mint_asset.key() == collateral_asset.token_mint @ LoanOfferError::InvalidCollateralMintAsset,
  )]
  pub collateral_mint_asset: Box<Account<'info, Mint>>,
  #[account(
    constraint = lend_mint_asset.key() == lend_asset.token_mint @ LoanOfferError::InvalidLendMintAsset,
  )]
  pub lend_mint_asset: Box<Account<'info, Mint>>,
  #[account(
    mut,
    constraint = borrower_ata_asset.amount >= collateral_amount @ LoanOfferError::NotEnoughAmount,
    associated_token::mint = collateral_mint_asset,
    associated_token::authority = borrower
  )]
  pub borrower_ata_asset: Box<Account<'info, TokenAccount>>,
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
  pub lend_asset: Box<Account<'info, Asset>>,
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
  pub collateral_asset: Box<Account<'info, Asset>>,
  #[account(
    init,
    payer = borrower,
    space = (DISCRIMINATOR as usize) + LoanOfferAccount::INIT_SPACE,
    seeds = [
      ENSO_SEED.as_ref(),
      LOAN_OFFER_ACCOUNT_SEED.as_ref(),
      borrower.key().as_ref(),
      offer_id.as_bytes(),
      crate::ID.key().as_ref()
    ],
    bump
  )]
  pub loan_offer: Box<Account<'info, LoanOfferAccount>>,
  #[account(
    mut,
    constraint = lend_offer.status == LendOfferStatus::Created @ LoanOfferError::LendOfferIsNotAvailable,
    seeds = [
      ENSO_SEED.as_ref(), 
      LEND_OFFER_ACCOUNT_SEED.as_ref(), 
      lender.key().as_ref(), 
      lend_offer_id.as_bytes(),
      crate::ID.key().as_ref(), 
    ],
    bump = lend_offer.bump
  )]
  pub lend_offer: Box<Account<'info, LendOfferAccount>>,
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
    init_if_needed,
    payer = borrower,
    associated_token::mint = collateral_mint_asset,
    associated_token::authority = vault_authority
  )]
  pub vault: Box<Account<'info, TokenAccount>>,
  pub lender: SystemAccount<'info>,
  #[account(
    constraint = lend_price_feed_account.key() == lend_asset.price_feed_account @ LoanOfferError::InvalidPriceFeedAccount,
  )]
  pub lend_price_feed_account: Box<Account<'info, PriceUpdateV2>>,
  #[account(
    constraint = collateral_price_feed_account.key() == collateral_asset.price_feed_account @ LoanOfferError::InvalidPriceFeedAccount,
  )]
  pub collateral_price_feed_account: Box<Account<'info, PriceUpdateV2>>,
  #[account(
    seeds = [
      ENSO_SEED.as_ref(), 
      SETTING_ACCOUNT_SEED.as_ref(),
      tier_id.as_bytes(), 
      crate::ID.key().as_ref(), 
    ],
    bump = setting_account.bump
  )]
  pub setting_account: Box<Account<'info, SettingAccount>>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}

impl<'info> CreateLoanOffer<'info> {
  pub fn create_loan_offer(
    &mut self,
    offer_id: String, 
    lend_offer_id: String, 
    tier_id: String, 
    collateral_amount: u64,
    interest: f64,
    bumps: &CreateLoanOfferBumps
  ) -> Result<()> {
    self.validate_lend_offer(interest)?;
    health_ratio::validate_health_ratio(HealthRatioParams {
      collateral_price_feed_account: &self.collateral_price_feed_account,
      collateral_amount,
      collateral_price_feed_id: self.collateral_asset.price_feed_id.clone(),
      collateral_max_price_age_seconds: self.collateral_asset.max_price_age_seconds,
      collateral_decimals: self.collateral_asset.decimals,
      lend_price_feed_account: &self.lend_price_feed_account,
      lend_amount: self.lend_offer.amount,
      lend_price_feed_id: self.lend_asset.price_feed_id.clone(),
      lend_max_price_age_seconds: self.lend_asset.max_price_age_seconds,
      lend_decimals: self.lend_asset.decimals,
    })?;

    self.deposit_collateral(collateral_amount)?;

    self.lend_offer.status = LendOfferStatus::Loaned;
    self.loan_offer.set_inner(LoanOfferAccount {
      tier_id,
      borrow_amount: self.lend_offer.amount,
      borrower: self.borrower.key(),
      borrower_fee_percent: self.setting_account.borrower_fee_percent,
      bump: bumps.loan_offer,
      collateral_amount,
      collateral_mint_token: self.collateral_mint_asset.key(),
      duration: self.lend_offer.duration,
      interest: self.lend_offer.interest,
      lend_mint_token: self.lend_offer.lend_mint_token.key(),
      lend_offer_id,
      lender: self.lend_offer.lender,
      lender_fee_percent: self.lend_offer.lender_fee_percent,
      offer_id,
      started_at: Clock::get()?.unix_timestamp,
      status: LoanOfferStatus::Matched,
      liquidating_at: None,
      liquidating_price: None,
      liquidated_tx: None,
      liquidated_price: None,
      request_withdraw_amount: None
    });

    self.emit_event_create_loan_offer()?;

    Ok(())
  }

  fn validate_lend_offer(&self, interest: f64) -> Result<()> {
    if self.lend_offer.interest != interest {
      return err!(LoanOfferError::CanNotCreateLoanCauseLendInterestUpdated);
    }

    Ok(())
  }

  fn deposit_collateral(&self, collateral_amount: u64) -> Result<()> {
    let cpi_context = CpiContext::new(self.token_program.to_account_info(), TransferChecked {
      from: self.borrower_ata_asset.to_account_info(),
      mint: self.collateral_mint_asset.to_account_info(),
      to: self.vault.to_account_info(),
      authority: self.borrower.to_account_info(),
    });

    transfer_checked(
      cpi_context,
      collateral_amount,
      self.collateral_mint_asset.decimals,
    )
  }

  fn emit_event_create_loan_offer(&self) -> Result<()> {
    emit!(LoanOfferCreateRequestEvent {
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
