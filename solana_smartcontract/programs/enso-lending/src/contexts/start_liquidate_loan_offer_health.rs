use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{
  common::constant::{
    ASSET_SEED, ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED, OPERATE_SYSTEM_PUBKEY, VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED
  }, health_ratio::{self, HealthRatioParams}, Asset, LiquidatingCollateralEvent, LoanOfferAccount, LoanOfferError, LoanOfferStatus, VaultAuthority, HOT_WALLET_PUBKEY, MIN_BORROW_HEALTH_RATIO
};

#[derive(Accounts)]
#[instruction(offer_id: String)]
pub struct StartLiquidateLoanOfferHealth<'info> {
  #[account(
    mut,
    constraint = system.key() == Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() @ LoanOfferError::InvalidSystem
  )]
  pub system: Signer<'info>,
  pub borrower: SystemAccount<'info>,
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
  #[account(
    constraint = collateral_mint_asset.key() == loan_offer.collateral_mint_token @ LoanOfferError::InvalidCollateralMintAsset,
  )]
  pub collateral_mint_asset: Account<'info, Mint>,
  #[account(
    constraint = lend_mint_asset.key() == loan_offer.lend_mint_token @ LoanOfferError::InvalidLendMintAsset,
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
    constraint = lend_price_feed_account.key() == lend_asset.price_feed_account @ LoanOfferError::InvalidPriceFeedAccount,
  )]
  pub lend_price_feed_account: Account<'info, PriceUpdateV2>,
  #[account(
    constraint = collateral_price_feed_account.key() == collateral_asset.price_feed_account @ LoanOfferError::InvalidPriceFeedAccount,
  )]
  pub collateral_price_feed_account: Account<'info, PriceUpdateV2>,
  #[account(
    mut,
    associated_token::mint = collateral_mint_asset,
    associated_token::authority = Pubkey::from_str(HOT_WALLET_PUBKEY).unwrap()
  )]
  pub hot_wallet_ata_collateral_asset: Account<'info, TokenAccount>,
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
  pub token_program: Program<'info, Token>,
}

impl<'info> StartLiquidateLoanOfferHealth<'info> {
  pub fn start_liquidate_loan_offer_health(&mut self) -> Result<()> {
    let loan_offer = &mut self.loan_offer;
    if loan_offer.status != LoanOfferStatus::FundTransferred {
      return err!(LoanOfferError::InvalidOfferStatus);
    }

    let (current_health_ratio, current_collateral_price, _) = health_ratio::get_health_ratio_and_assets_price(HealthRatioParams {
      collateral_price_feed_account: &self.collateral_price_feed_account,
      collateral_amount: loan_offer.collateral_amount,
      collateral_price_feed_id: self.collateral_asset.price_feed_id.clone(),
      collateral_max_price_age_seconds: self.collateral_asset.max_price_age_seconds,
      collateral_decimals: self.collateral_asset.decimals,
      lend_price_feed_account: &self.lend_price_feed_account,
      lend_amount: loan_offer.borrow_amount,
      lend_price_feed_id: self.lend_asset.price_feed_id.clone(),
      lend_max_price_age_seconds: self.lend_asset.max_price_age_seconds,
      lend_decimals: self.lend_asset.decimals,
    });

    if current_health_ratio < MIN_BORROW_HEALTH_RATIO {
      loan_offer.liquidating_price = Some(current_collateral_price);
      loan_offer.liquidating_at = Some(Clock::get().unwrap().unix_timestamp);
      loan_offer.status = LoanOfferStatus::Liquidating;
  
      self.transfer_collateral_to_hot_wallet()?;

      self.emit_event_start_liquidate_contract()?;

      Ok(())
    } else {
      return err!(LoanOfferError::HealthRatioInvalid);
    }
  }

  fn transfer_collateral_to_hot_wallet(&mut self) -> Result<()> {
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
        to: self.hot_wallet_ata_collateral_asset.to_account_info(),
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

  fn emit_event_start_liquidate_contract(&self) -> Result<()> {
    emit!(LiquidatingCollateralEvent {
      offer_id: self.loan_offer.offer_id.clone(),
      liquidating_at: self.loan_offer.liquidating_at,
      liquidating_price: self.loan_offer.liquidating_price,
    });

    Ok(())
  }
}