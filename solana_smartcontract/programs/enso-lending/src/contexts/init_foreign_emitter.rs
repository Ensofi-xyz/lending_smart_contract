use std::str::FromStr;

pub use anchor_lang::prelude::*;
use wormhole_anchor_sdk::wormhole::SEED_PREFIX_EMITTER;

use crate::{common::{constant::OPERATE_SYSTEM_PUBKEY, ENSO_SEED}, EmitterAccountError, ForeignEmitter, InitForeignEmitterEvent, DISCRIMINATOR};

#[derive(Accounts)]
#[instruction(chain: u16, address: String)]
pub struct InitForeignEmitter<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  #[account(
    init,
    payer = owner,
    space = (DISCRIMINATOR as usize) + ForeignEmitter::INIT_SPACE,
    seeds = [
      ENSO_SEED.as_ref(), 
      SEED_PREFIX_EMITTER.as_ref(),
      &chain.to_be_bytes(),
      crate::ID.key().as_ref(), 
    ],
    bump
  )]
  pub foreign_emitter: Account<'info, ForeignEmitter>,
  pub system_program: Program<'info, System>,
}

impl<'info> InitForeignEmitter<'info> {
    pub fn init_foreign_emitter_account(
      &mut self, 
      bumps: &InitForeignEmitterBumps,
      chain: u16,
      address: String
    ) -> Result<()> {

      if self.owner.key() != Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() {
        return err!(EmitterAccountError::InvalidOwner)?;
      }

      self.foreign_emitter.set_inner(ForeignEmitter {
        chain,
        address,
        bump: bumps.foreign_emitter
      });

      self.emit_init_foreign_emitter_event()?;

      Ok(())
    }

    fn emit_init_foreign_emitter_event(&mut self) -> Result<()> {
      emit!(InitForeignEmitterEvent {
          chain: self.foreign_emitter.chain,
          address: self.foreign_emitter.address.clone(),
      });
            
      Ok(())
  }
}