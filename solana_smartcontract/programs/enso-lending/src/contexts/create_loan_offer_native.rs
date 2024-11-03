use anchor_lang::{prelude::*, solana_program::{program::invoke_signed, system_instruction}};
use anchor_spl::token::{Mint, Token};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{
  common::{
    ASSET_SEED, 
    ENSO_SEED, 
    LEND_OFFER_ACCOUNT_SEED, 
    LOAN_OFFER_ACCOUNT_SEED, 
    SETTING_ACCOUNT_SEED
  }, 
  health_ratio::{self, HealthRatioParams}, 
  Asset, 
  LendOfferAccount, 
  LendOfferStatus, 
  LoanOfferAccount, 
  LoanOfferCreateRequestEvent, 
  LoanOfferError, 
  LoanOfferStatus, 
  SettingAccount, 
  DISCRIMINATOR, 
};

#[derive(Accounts)]
#[instruction(
  offer_id: String, 
  lend_offer_id: String, 
  tier_id: String, 
  collateral_amount: u64
)]
pub struct CreateLoanOfferNative<'info> {
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
  pub loan_offer: Account<'info, LoanOfferAccount>,
  /// CHECK: This account is used to check the validate of lend offer account
  pub lender: AccountInfo<'info>,
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
  pub lend_offer: Account<'info, LendOfferAccount>,
  #[account(
    constraint = lend_price_feed_account.key() == lend_asset.price_feed_account @ LoanOfferError::InvalidPriceFeedAccount,
  )]
  pub lend_price_feed_account: Account<'info, PriceUpdateV2>,
  #[account(
    constraint = collateral_price_feed_account.key() == collateral_asset.price_feed_account @ LoanOfferError::InvalidPriceFeedAccount,
  )]
  pub collateral_price_feed_account: Account<'info, PriceUpdateV2>,
  #[account(
    seeds = [
        ENSO_SEED.as_ref(), 
        SETTING_ACCOUNT_SEED.as_ref(),
        tier_id.as_bytes(), 
        crate::ID.key().as_ref(), 
    ],
    bump = setting_account.bump
  )]
  pub setting_account: Account<'info, SettingAccount>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}

impl<'info> CreateLoanOfferNative<'info> {
  pub fn create_loan_offer_native(
    &mut self,
    bumps: &CreateLoanOfferNativeBumps,
    offer_id: String, 
    lend_offer_id: String, 
    tier_id: String, 
    collateral_amount: u64,
    interest: f64
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
      collateral_mint_token: self.collateral_asset.token_mint,
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

  fn validate_lend_offer(&self, interest: f64) -> Result<()> {
    if self.lend_offer.interest != interest {
      return err!(LoanOfferError::CanNotCreateLoanCauseLendInterestUpdated);
    }

    Ok(())
  }

  fn deposit_collateral(&self, collateral_amount: u64) -> Result<()> {
    let transfer_instruction = system_instruction::transfer(
      &self.borrower.key(),
      &self.loan_offer.key(),
      collateral_amount
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