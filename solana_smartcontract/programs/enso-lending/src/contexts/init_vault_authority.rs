use anchor_lang::prelude::*;

use crate::{VaultAuthority, ENSO_SEED, VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED};

#[derive(Accounts)]
pub struct InitVaultAuthority<'info> {
  #[account(mut)]
  pub borrower: Signer<'info>,
  #[account(
    init_if_needed,
    payer = borrower,
    space = 8 + VaultAuthority::INIT_SPACE,
    seeds = [
      ENSO_SEED.as_ref(), 
      borrower.key().as_ref(),
      VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED.as_ref(), 
      crate::ID.key().as_ref(), 
    ],
    bump
  )]
  pub vault_authority: Account<'info, VaultAuthority>,
  pub system_program: Program<'info, System>,
}

impl<'info> InitVaultAuthority<'info> {
  pub fn init_vault_authority(&mut self, bumps: &InitVaultAuthorityBumps) -> Result<()> {
    self.vault_authority.set_inner(VaultAuthority {
      initializer: self.borrower.key(),
      bump: bumps.vault_authority
    });

    Ok(())
  }
}