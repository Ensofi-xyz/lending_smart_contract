use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{
  Asset, 
  EditAssetEvent, 
  SettingAccountError, 
  ASSET_SEED, 
  ENSO_SEED, 
  OPERATE_SYSTEM_PUBKEY
};

#[derive(Accounts)]
pub struct EditAsset<'info> {
  #[account(
    mut,
    constraint = owner.key() == Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() @ SettingAccountError::InvalidOwner
  )]
  pub owner: Signer<'info>,
  pub token_mint: Account<'info, Mint>,
  #[account(
    mut,
    seeds = [
      ENSO_SEED.as_ref(),
      ASSET_SEED.as_ref(),
      token_mint.key().as_ref(),
      crate::ID.key().as_ref()
    ],
    bump
  )]
  pub asset: Account<'info, Asset>,
  pub price_feed_account: Option<Account<'info, PriceUpdateV2>>
}

impl<'info> EditAsset<'info> {
  pub fn edit_asset(
    &mut self, 
    name: Option<String>, 
    is_lend: Option<bool>, 
    is_collateral: Option<bool>, 
    price_feed_id: Option<String>,
    max_price_age_seconds: Option<u64>,
    token_address: Option<String>
  ) -> Result<()> {
    let asset = &mut self.asset;

    if let Some(name) = name {
      asset.name = name;
    }
    if let Some(is_lend) = is_lend {
      asset.is_lend = is_lend;
    }
    if let Some(is_collateral) = is_collateral {
      asset.is_collateral = is_collateral;
    }
    if let Some(price_feed_id) = price_feed_id {
      asset.price_feed_id = price_feed_id;
    }
    if let Some(max_price_age_seconds) = max_price_age_seconds {
      asset.max_price_age_seconds = max_price_age_seconds;
    }
    if let Some(price_feed_account) = &self.price_feed_account {
      asset.price_feed_account = price_feed_account.key();
    }

    if let Some(token_address) = token_address {
      asset.token_address = Some(token_address);
    }

    self.emit_edit_asset_event()?;

    Ok(())
  }

  fn emit_edit_asset_event(&mut self) -> Result<()> {
    emit!(EditAssetEvent {
      name: self.asset.name.clone(),
      token_mint: self.asset.token_mint.key(),
      decimals: self.asset.decimals,
      is_collateral: self.asset.is_collateral,
      is_lend: self.asset.is_lend,
      max_price_age_seconds: self.asset.max_price_age_seconds,
      price_feed_account: self.asset.price_feed_account.key(),
      price_feed_id: self.asset.price_feed_id.clone(),
      bump: self.asset.bump
    });
          
    Ok(())
  }
}