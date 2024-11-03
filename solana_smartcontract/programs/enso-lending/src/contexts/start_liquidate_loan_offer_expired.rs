use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{
  common::constant::{
    ENSO_SEED, 
    LOAN_OFFER_ACCOUNT_SEED, 
    OPERATE_SYSTEM_PUBKEY,
    VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED
  }, LiquidatingCollateralEvent, LoanOfferAccount, LoanOfferError, LoanOfferStatus, VaultAuthority, HOT_WALLET_PUBKEY, 
};

#[derive(Accounts)]
#[instruction(offer_id: String)]
pub struct StartLiquidateLoanOfferExpired<'info> {
  #[account(
    mut,
    constraint = system.key() == Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() @ LoanOfferError::InvalidSystem
  )]
  pub system: Signer<'info>,
  pub borrower: SystemAccount<'info>,
  #[account(
    constraint = collateral_mint_asset.key() == loan_offer.collateral_mint_token @ LoanOfferError::InvalidCollateralMintAsset,
  )]
  pub collateral_mint_asset: Account<'info, Mint>,
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

impl<'info> StartLiquidateLoanOfferExpired<'info> {
  pub fn start_liquidate_loan_offer_expired(&mut self) -> Result<()> {
    self.validate_expired_loan_offer()?;
    
    let loan_offer = &mut self.loan_offer;
    if loan_offer.status != LoanOfferStatus::FundTransferred {
      return err!(LoanOfferError::InvalidOfferStatus);
    }

    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    loan_offer.liquidating_at = Some(current_timestamp);
    loan_offer.status = LoanOfferStatus::Liquidating;

    self.transfer_collateral_to_hot_wallet()?;

    self.emit_event_start_liquidate_contract()?;

    Ok(())
  }

  fn emit_event_start_liquidate_contract(&self) -> Result<()> {
    emit!(LiquidatingCollateralEvent {
      offer_id: self.loan_offer.offer_id.clone(),
      liquidating_at: self.loan_offer.liquidating_at,
      liquidating_price: self.loan_offer.liquidating_price,
    });

    Ok(())
  }

  fn validate_expired_loan_offer(&self) -> Result<()> {
    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    let end_borrowed_loan_offer = self.loan_offer.started_at + self.loan_offer.duration as i64;

    if current_timestamp < end_borrowed_loan_offer {
      return err!(LoanOfferError::LoanOfferNotExpired);
    }

    Ok(())
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


}