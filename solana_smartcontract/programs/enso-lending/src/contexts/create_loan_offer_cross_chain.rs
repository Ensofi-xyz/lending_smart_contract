use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use wormhole_anchor_sdk::wormhole::{self, program::Wormhole};

use crate::{
  common::{ENSO_SEED, LEND_OFFER_ACCOUNT_SEED, SETTING_ACCOUNT_SEED}, Asset, ForeignChain, LendOfferAccount, LendOfferStatus, LoanOfferAccount, LoanOfferCreateRequestEvent, LoanOfferError, LoanOfferStatus, SettingAccount, WormholeMessage, ASSET_SEED, CREATE_LOAN_OFFER_CROSS_CHAIN_FUNCTION, DISCRIMINATOR, LOAN_OFFER_ACCOUNT_SEED, POSTED_TIMESTAMP_THRESHOLD
};
use crate::utils::vaa;

#[derive(Accounts)]
#[instruction(
  tier_id: String,
  loan_offer_id: String, 
  lend_offer_id: String,
  vaa_hash: [u8; 32],
)]
pub struct CreateLoanOfferCrossChain<'info> {
  #[account(mut)]
  pub signer: Signer<'info>,
  #[account(
    constraint = collateral_mint_asset.key() == collateral_asset.token_mint @ LoanOfferError::InvalidCollateralMintAsset,
  )]
  pub collateral_mint_asset: Box<Account<'info, Mint>>,
  #[account(
    constraint = lend_mint_asset.key() == lend_asset.token_mint @ LoanOfferError::InvalidLendMintAsset,
  )]
  pub lend_mint_asset: Box<Account<'info, Mint>>,
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
    payer = signer,
    space = (DISCRIMINATOR as usize) + LoanOfferAccount::INIT_SPACE,
    seeds = [
      ENSO_SEED.as_ref(),
      LOAN_OFFER_ACCOUNT_SEED.as_ref(),
      borrower.key().as_ref(),
      loan_offer_id.as_bytes(),
      crate::ID.key().as_ref()
    ],
    bump
    )] 
  pub loan_offer:Box<Account<'info, LoanOfferAccount>>,
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
  pub lender: SystemAccount<'info>,
  pub borrower: SystemAccount<'info>,
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
  #[account(
    seeds = [
      wormhole::SEED_PREFIX_POSTED_VAA,
      &vaa_hash
    ],
    bump,
    seeds::program = wormhole_program.key
  )]
  /// signatures and posted the account data here. Read-only.
  pub posted: Account<'info, wormhole::PostedVaa<WormholeMessage>>,
  #[account(
    mut,
    constraint = collateral_asset.chain_id == foreign_chain.chain_id @ LoanOfferError::InvalidChainId
  )]
  pub foreign_chain: Account<'info, ForeignChain>,
  pub wormhole_program: Program<'info, Wormhole>,
  pub system_program: Program<'info, System>,
}

impl<'info> CreateLoanOfferCrossChain<'info> {
  pub fn create_loan_offer_cross_chain(
    &mut self,
    bumps: &CreateLoanOfferCrossChainBumps,
    tier_id: String,
    loan_offer_id: String, 
    lend_offer_id: String, 
    _vaa_hash: [u8; 32],
  ) -> Result<()> {
    self.validate_posted_vaa()?;

    let posted_vaa = &self.posted.clone().into_inner();
    let WormholeMessage::Message { payload } = posted_vaa.data();
    let ( 
      _target_chain,
      _target_address,
      target_function,
      posted_tier_id,
      posted_lend_offer_id,
      lend_amount,
      collateral_amount,
      collateral_address,
      borrower_address,

    ) = self.parse_create_loan_payload(&payload).unwrap();

    let _ = self.verify_payload_message_data(
      target_function, 
      posted_lend_offer_id, 
      lend_amount,
      posted_tier_id,
      tier_id.clone(),
      borrower_address,
      collateral_address
    )?; 

    self.lend_offer.status = LendOfferStatus::Loaned;
    self.loan_offer.set_inner(LoanOfferAccount {
      tier_id: tier_id.clone(),
      offer_id: loan_offer_id,
      borrow_amount: self.lend_offer.amount,
      borrower: self.borrower.key(),
      borrower_fee_percent: self.setting_account.borrower_fee_percent,
      bump: bumps.loan_offer,
      collateral_amount,
      collateral_mint_token: self.collateral_mint_asset.key(),
      lend_offer_id,
      interest: self.lend_offer.interest,
      lender_fee_percent: self.lend_offer.lender_fee_percent,
      duration: self.lend_offer.duration,
      lender: self.lend_offer.lender,
      status: LoanOfferStatus::Matched,
      lend_mint_token: self.lend_offer.lend_mint_token.key(),
      started_at: Clock::get()?.unix_timestamp,
      liquidating_at: None,
      liquidating_price: None,
      liquidated_tx: None,
      liquidated_price: None,
      request_withdraw_amount: None,
    });

    self.emit_event_create_loan_offer_cross_chain()?;

    Ok(())
  }

  fn validate_posted_vaa(&self) -> Result<()> {
    let posted_chain_id = self.posted.meta.emitter_chain;
    let chain_id = self.foreign_chain.chain_id;
    let posted_emitter_address = self.posted.meta.emitter_address;
    let emitter_address = self.foreign_chain.emitter_address.clone();
    let posted_timestamp: u32 = self.posted.meta.timestamp;
    let current_timestamp: u32 = Clock::get()?.unix_timestamp.try_into().unwrap();

    if posted_timestamp + POSTED_TIMESTAMP_THRESHOLD < current_timestamp {
      return err!(LoanOfferError::PostedVaaExpired);
    }

    Ok(vaa::validate_posted_vaa(
        posted_chain_id,
        chain_id,
        posted_emitter_address,
        emitter_address,
    )?)
  }

  fn parse_create_loan_payload(
    &self,
    posted_vaa: &Vec<u8>,
  ) -> Result<(u16, String, String, String, String, u64, u64, String, String)> {
      Ok(vaa::parse_create_loan_payload(posted_vaa)?)
  }

  fn verify_payload_message_data(
    &self,
    target_function: String,
    lend_offer_id: String,
    lend_amount: u64,
    posted_tier_id: String,
    tier_id: String,
    borrower_address: String,
    collateral_address: String,
  ) -> Result<()> {
    if target_function != CREATE_LOAN_OFFER_CROSS_CHAIN_FUNCTION {
      return err!(LoanOfferError::InvalidTargetFunction);
    }

    if lend_offer_id != self.lend_offer.offer_id {
      return err!(LoanOfferError::LendOfferIdNotMatch);
    }

    if lend_amount < self.lend_offer.amount {
      return err!(LoanOfferError::InvalidLendOfferAmount);
    }

    if posted_tier_id != tier_id {
      return err!(LoanOfferError::TierIdNotMatch);
    }

    if borrower_address != self.borrower.key().to_string() {
      return err!(LoanOfferError::InvalidBorrower);
    }

    if self.collateral_asset.token_address != Some(collateral_address) {
      return err!(LoanOfferError::InvalidAssetAccount);
    }

    Ok(())
  }

  fn emit_event_create_loan_offer_cross_chain(
    &self,
  ) -> Result<()> {
    emit!(LoanOfferCreateRequestEvent {
      offer_id: self.loan_offer.offer_id.clone(),
      collateral_mint_token: self.collateral_mint_asset.key(),
      tier_id: self.loan_offer.tier_id.clone(),
      lend_offer_id: self.loan_offer.lend_offer_id.clone(),
      interest: self.loan_offer.interest,
      borrow_amount: self.loan_offer.borrow_amount,
      lender_fee_percent: self.loan_offer.lender_fee_percent,
      duration: self.loan_offer.duration,
      lend_mint_token: self.lend_mint_asset.key(),
      lender: self.lender.key(),
      borrower: self.borrower.key(),
      collateral_amount: self.loan_offer.collateral_amount,
      status: self.loan_offer.status,
      borrower_fee_percent: self.loan_offer.borrower_fee_percent,
      started_at: self.loan_offer.started_at,
    });

    Ok(())
  }
}