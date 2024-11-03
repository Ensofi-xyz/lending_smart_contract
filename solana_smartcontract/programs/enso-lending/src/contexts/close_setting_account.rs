use anchor_lang::prelude::*;
use anchor_lang::__private::CLOSED_ACCOUNT_DISCRIMINATOR;
use std::io::{Cursor, Write};
use std::ops::DerefMut;

use crate::common::{CloseSettingAccountEvent, constant::{ ENSO_SEED, SETTING_ACCOUNT_SEED }};
use crate::{SettingAccount, SettingAccountError};

#[derive(Accounts)]
#[instruction(tier_id: String)]
pub struct CloseSettingAccount<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  /// CHECK: This is the account used to make a seeds to create ata account for transfer asset from lender to how wallet
  #[account(
    mut,
    has_one = owner,
    constraint = setting_account.tier_id == tier_id @ SettingAccountError::InvalidTierId,
    seeds = [
      ENSO_SEED.as_ref(), 
      SETTING_ACCOUNT_SEED.as_ref(),
      tier_id.as_bytes(), 
      crate::ID.key().as_ref(), 
    ],
    bump
  )]
  pub setting_account: Account<'info, SettingAccount>,
  pub system_program: Program<'info, System>,
}

impl<'info> CloseSettingAccount<'info> {
  pub fn close_setting_account(&mut self, tier_id: String) -> Result<()>  {
    let dest_starting_lamports = self.owner.lamports();
    let setting_account = self.setting_account.to_account_info();

    **self.owner.lamports.borrow_mut() = dest_starting_lamports
      .checked_add(setting_account.lamports())
      .unwrap();
    **setting_account.lamports.borrow_mut() = 0;

    let mut data = setting_account.try_borrow_mut_data()?;
    for byte in data.deref_mut().iter_mut() {
      *byte = 0;
    }

    let dst: &mut [u8] = &mut data;
    let mut cursor = Cursor::new(dst);
    cursor.write_all(&CLOSED_ACCOUNT_DISCRIMINATOR).unwrap();

    self.emit_event_close_setting_account(tier_id)?;

    Ok(())
  }

  fn emit_event_close_setting_account(
    &mut self,
    tier_id: String,
  ) -> Result<()> {
    emit!(CloseSettingAccountEvent { tier_id });


    Ok(())
  }
}