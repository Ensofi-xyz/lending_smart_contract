use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{
  Asset, 
  InitAssetEvent, 
  SettingAccountError, 
  ASSET_SEED, 
  DISCRIMINATOR, 
  ENSO_SEED, 
  OPERATE_SYSTEM_PUBKEY
};

#[derive(Accounts)]
pub struct InitAsset<'info> {
  #[account(
    mut,
    constraint = owner.key() == Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() @ SettingAccountError::InvalidOwner
  )]
  pub owner: Signer<'info>,
  pub token_mint: Account<'info, Mint>,
  #[account(
    init,
    payer = owner,
    space = (DISCRIMINATOR as usize) + Asset::INIT_SPACE,
    seeds = [
      ENSO_SEED.as_ref(),
      ASSET_SEED.as_ref(),
      token_mint.key().as_ref(),
      crate::ID.key().as_ref()
    ],
    bump
  )]
  pub asset: Account<'info, Asset>,
  pub price_feed_account: Account<'info, PriceUpdateV2>,
  pub system_program: Program<'info, System>,
}

impl<'info> InitAsset<'info>{
    pub fn init_asset(
      &mut self, 
      name: String, 
      is_lend: bool, 
      is_collateral: bool, 
      price_feed_id: String,
      max_price_age_seconds: u64,
      token_address: Option<String>,
      chain_id: u16,
      bumps: &InitAssetBumps
    ) -> Result<()> {
      self.asset.set_inner(Asset {
        name,
        token_mint: self.token_mint.key(),
        token_address,
        decimals: self.token_mint.decimals,
        is_collateral,
        is_lend,
        max_price_age_seconds,
        price_feed_account: self.price_feed_account.key(),
        price_feed_id,
        chain_id,
        bump: bumps.asset
      });

      self.emit_init_asset_event()?;

      Ok(())
    }

    fn emit_init_asset_event(&mut self) -> Result<()> {
      emit!(InitAssetEvent {
        name: self.asset.name.clone(),
        token_address: self.asset.token_address.clone(),
        token_mint: self.asset.token_mint.key(),
        decimals: self.asset.decimals,
        is_collateral: self.asset.is_collateral,
        is_lend: self.asset.is_lend,
        max_price_age_seconds: self.asset.max_price_age_seconds,
        price_feed_account: self.asset.price_feed_account.key(),
        price_feed_id: self.asset.price_feed_id.clone(),
        chain_id: self.asset.chain_id,
        bump: self.asset.bump
      });
            
      Ok(())
    }
}