use std::str::FromStr;

pub use anchor_lang::prelude::*;
use wormhole_anchor_sdk::wormhole::SEED_PREFIX_EMITTER;

use crate::{common::{constant::OPERATE_SYSTEM_PUBKEY, ENSO_SEED}, EmitterAccountError, ForeignChain, InitForeignEmitterEvent, DISCRIMINATOR};

#[derive(Accounts)]
#[instruction(chain_id: u16, chain_address: String, emitter_address: String)]
pub struct InitForeignEmitter<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  #[account(
    init,
    payer = owner,
    space = (DISCRIMINATOR as usize) + ForeignChain::INIT_SPACE,
    seeds = [
      ENSO_SEED.as_ref(), 
      SEED_PREFIX_EMITTER.as_ref(),
      &chain_id.to_be_bytes(),
      crate::ID.key().as_ref(), 
    ],
    bump
  )]
  pub foreign_chain: Account<'info, ForeignChain>,
  pub system_program: Program<'info, System>,
}

impl<'info> InitForeignEmitter<'info> {
    pub fn init_foreign_chain_account(
      &mut self, 
      bumps: &InitForeignEmitterBumps,
      chain_id: u16,
      chain_address: String,
      emitter_address: String
    ) -> Result<()> {

      if self.owner.key() != Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() {
        return err!(EmitterAccountError::InvalidOwner)?;
      }

      self.foreign_chain.set_inner(ForeignChain {
        chain_id,
        chain_address,
        emitter_address,
        bump: bumps.foreign_chain
      });

      self.emit_init_foreign_chain_event()?;

      Ok(())
    }

    fn emit_init_foreign_chain_event(&mut self) -> Result<()> {
      emit!(InitForeignEmitterEvent {
          chain_id: self.foreign_chain.chain_id,
          chain_address: self.foreign_chain.emitter_address.clone(),
          emitter_address: self.foreign_chain.emitter_address.clone()
      });
            
      Ok(())
  }
}