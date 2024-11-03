use std::str::FromStr;

pub use anchor_lang::prelude::*;
use wormhole_anchor_sdk::wormhole::SEED_PREFIX_EMITTER;

use crate::{common::{constant::OPERATE_SYSTEM_PUBKEY, ENSO_SEED}, EmitterAccountError, WormholeEmitter, DISCRIMINATOR};

#[derive(Accounts)]
#[instruction(chain: u16, address: String)]
pub struct InitWormholeEmitter<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  #[account(
    init,
    payer = owner,
    space = (DISCRIMINATOR as usize) + WormholeEmitter::INIT_SPACE,
    seeds = [
      ENSO_SEED.as_ref(), 
      SEED_PREFIX_EMITTER.as_ref(),
      &chain.to_be_bytes(),
      crate::ID.key().as_ref(), 
    ],
    bump
  )]
  pub wormhole_emitter: Account<'info, WormholeEmitter>,
  pub system_program: Program<'info, System>,
}

impl<'info> InitWormholeEmitter<'info> {
    pub fn init_wormhole_emitter_account(
      &mut self, 
      bumps: &InitWormholeEmitterBumps,
      chain: u16,
      address: String
    ) -> Result<()> {

      if self.owner.key() != Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() {
        return err!(EmitterAccountError::InvalidOwner)?;
      }

      self.wormhole_emitter.set_inner(WormholeEmitter {
        chain,
        address,
        bump: bumps.wormhole_emitter
      });

      Ok(())
    }
}