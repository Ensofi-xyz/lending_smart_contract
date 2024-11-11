use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use wormhole_anchor_sdk::wormhole::{self, program::Wormhole};


use crate::{Asset, ASSET_SEED};
use crate::{
  common::ENSO_SEED, ForeignChain, LoanOfferAccount, LoanOfferError, LoanOfferStatus, UpdateWithdrawCollateralCrossChainEvent, WormholeMessage, LOAN_OFFER_ACCOUNT_SEED, UPDATE_WITHDRAW_COLLATERAL_CROSS_CHAIN_FUNCTION
};
use crate::utils::vaa;

#[derive(Accounts)]
#[instruction(
  loan_offer_id: String, 
  vaa_hash: [u8; 32]
)]
pub struct UpdateWithdrawCollateralCrossChain<'info> {
  #[account(mut)]
  pub signer: Signer<'info>,

  pub borrower: SystemAccount<'info>,

   #[account(
    constraint = collateral_mint_asset.key() == collateral_asset.token_mint @ LoanOfferError::InvalidCollateralMintAsset,
  )]
  pub collateral_mint_asset: Box<Account<'info, Mint>>,

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
    mut,
    constraint = loan_offer.status == LoanOfferStatus::Matched || loan_offer.status == LoanOfferStatus::FundTransferred 
    @ LoanOfferError::InvalidLoanOffer,
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

impl<'info>UpdateWithdrawCollateralCrossChain<'info> {
  pub fn update_withdraw_collateral_cross_chain(
    &mut self,
    loan_offer_id: String, 
    _vaa_hash: [u8; 32],
  ) -> Result<()> {
    self.validate_posted_vaa()?;

    let posted_vaa = &self.posted.clone().into_inner();
    let WormholeMessage::Message { payload } = posted_vaa.data();
    let ( 
      target_chain,
      chain_address,
      target_function,
      posted_lend_offer_id,
      withdraw_amount,
      remaining_collateral_amount,
      collateral_address,
      borrower_address
    ) = self.parse_withdraw_collateral_payload(&payload).unwrap();

    let _ = self.verify_payload_message_data(
      posted_lend_offer_id.clone(), 
      borrower_address,
      withdraw_amount,
      remaining_collateral_amount,
      target_function,
      collateral_address.clone()
    )?; 

    self.loan_offer.collateral_amount = remaining_collateral_amount;

    self.emit_event_update_withdraw_collateral_cross_chain(
      loan_offer_id,
      chain_address,
      target_chain,
      withdraw_amount,
      remaining_collateral_amount,
      collateral_address,
    )?;

    Ok(())
  }

  fn validate_posted_vaa(&self) -> Result<()> {
    let posted_chain_id = self.posted.meta.emitter_chain;
    let chain_id = self.foreign_chain.chain_id;
    let posted_emitter_address = self.posted.meta.emitter_address;
    let emitter_address = self.foreign_chain.emitter_address.clone();

    Ok(vaa::validate_posted_vaa(
      posted_chain_id,
      chain_id,
      posted_emitter_address,
      emitter_address,
    )?)
  }

  fn parse_withdraw_collateral_payload(
    &self,
    posted_vaa: &Vec<u8>,
  ) -> Result<(u16, String, String, String, u64, u64, String, String)> {
      Ok(vaa::parse_withdraw_collateral_payload(posted_vaa)?)
  }

  fn verify_payload_message_data(
    &self,
    lend_offer_id: String,
    borrower_address: String,
    withdraw_amount: u64,
    remaining_collateral_amount: u64,
    target_function: String,
    collateral_address: String
  ) -> Result<()> {
    if lend_offer_id != self.loan_offer.lend_offer_id {
      return err!(LoanOfferError::InvalidLoanOffer);
    }

    if borrower_address != self.borrower.key().to_string() {
      return err!(LoanOfferError::InvalidBorrower);
    }

    if target_function != UPDATE_WITHDRAW_COLLATERAL_CROSS_CHAIN_FUNCTION {
      return err!(LoanOfferError::InvalidTargetFunction);
    }

    if self.collateral_asset.token_address != Some(collateral_address) {
      return err!(LoanOfferError::InvalidAssetAccount);
    }

    if withdraw_amount + remaining_collateral_amount != self.loan_offer.collateral_amount {
      return err!(LoanOfferError::WithdrawAmountNotMatch);
    }

    Ok(())
  }

  fn emit_event_update_withdraw_collateral_cross_chain(
    &self,
    loan_offer_id: String,
    chain_address: String,
    target_chain: u16,
    withdraw_amount: u64,
    remaining_collateral_amount: u64,
    collateral_address: String
  ) -> Result<()> {
    emit!(UpdateWithdrawCollateralCrossChainEvent {
      target_chain,
      chain_address,
      loan_offer_id,
      withdraw_amount,
      remaining_collateral_amount,
      collateral_address,
      borrower: self.borrower.key(),
    });

    Ok(())
  }
}