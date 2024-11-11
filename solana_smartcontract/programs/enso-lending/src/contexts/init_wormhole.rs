use std::str::FromStr;

pub use anchor_lang::prelude::*;
use wormhole_anchor_sdk::wormhole::{self, program::Wormhole};

use crate::{WormholeConfig, WormholeEmitter, WormholeError, WormholeMessage, OPERATE_SYSTEM_PUBKEY, WORMHOLE_SENT_SEED};

#[derive(Accounts)]
pub struct InitWormhole<'info> {
  #[account(
    mut,
    constraint = owner.key() == Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() @ WormholeError::InvalidOwner
  )]
  pub owner: Signer<'info>,

  #[account(
    init,
    payer = owner,
    seeds = [WormholeConfig::SEED_PREFIX],
    bump,
    space = WormholeConfig::MAXIMUM_SIZE,

  )]
  pub wormhole_config: Account<'info, WormholeConfig>,

  pub wormhole_program: Program<'info, Wormhole>,

  #[account(
    mut,
    seeds = [wormhole::BridgeData::SEED_PREFIX],
    bump,
    seeds::program = wormhole_program.key,
  )]
  pub wormhole_bridge: Account<'info, wormhole::BridgeData>,

  #[account(
    mut,
    seeds = [wormhole::FeeCollector::SEED_PREFIX],
    bump,
    seeds::program = wormhole_program.key
  )]
  pub wormhole_fee_collector: Account<'info, wormhole::FeeCollector>,

  #[account(
    init,
    payer = owner,
    seeds = [WormholeEmitter::SEED_PREFIX],
    bump,
    space = WormholeEmitter::MAXIMUM_SIZE
  )]
  pub wormhole_emitter: Account<'info, WormholeEmitter>,

  #[account(
    mut,
    seeds = [
        wormhole::SequenceTracker::SEED_PREFIX,
        wormhole_emitter.key().as_ref()
    ],
    bump,
    seeds::program = wormhole_program.key
  )]
  /// CHECK: Emitter's sequence account
  pub wormhole_sequence: UncheckedAccount<'info>,

  #[account(
    mut,
    seeds = [
        WORMHOLE_SENT_SEED,
        &wormhole::INITIAL_SEQUENCE.to_le_bytes()[..]
    ],
    bump,
  )]
  /// CHECK: Wormhole message account
  pub wormhole_message: UncheckedAccount<'info>,
  pub clock: Sysvar<'info, Clock>,
  pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
}

impl<'info> InitWormhole<'info> {
  pub fn init_wormhole(&mut self, bumps: &InitWormholeBumps,) -> Result<()> {
    let wormhole_config = &mut self.wormhole_config;

    wormhole_config.owner = self.owner.key();
    {
      let wormhole = &mut wormhole_config.wormhole;

      wormhole.bridge = self.wormhole_bridge.key();

      wormhole.fee_collector = self.wormhole_fee_collector.key();

      wormhole.sequence = self.wormhole_sequence.key();
    }

    wormhole_config.batch_id = 0;

    wormhole_config.finality = wormhole::Finality::Finalized as u8;

    self.wormhole_emitter.bump = bumps.wormhole_emitter;

    {
      let fee = self.wormhole_bridge.fee();
      if fee > 0 {
        solana_program::program::invoke(
          &solana_program::system_instruction::transfer(
            &self.owner.key(),
            &self.wormhole_fee_collector.key(),
            fee,
          ),
          &self.to_account_infos(),
        )?;
      }

      let wormhole_emitter = &self.wormhole_emitter;
      let wormhole_config = &self.wormhole_config;

      let mut payload: Vec<u8> = Vec::new();
      WormholeMessage::serialize(
        &WormholeMessage::Message {
          // This is a message with no payload.
          payload: vec![1, 2, 3, 4, 5],
        },
        &mut payload,
      )?;

      wormhole::post_message(
        CpiContext::new_with_signer(
          self.wormhole_program.to_account_info(),
          wormhole::PostMessage {
              config: self.wormhole_bridge.to_account_info(),
              message: self.wormhole_message.to_account_info(),
              emitter: wormhole_emitter.to_account_info(),
              sequence: self.wormhole_sequence.to_account_info(),
              payer: self.owner.to_account_info(),
              fee_collector: self.wormhole_fee_collector.to_account_info(),
              clock: self.clock.to_account_info(),
              rent: self.rent.to_account_info(),
              system_program: self.system_program.to_account_info(),
            },
          &[
            &[
              WORMHOLE_SENT_SEED,
              &wormhole::INITIAL_SEQUENCE.to_le_bytes()[..],
              &[bumps.wormhole_message],
            ],
            &[wormhole::SEED_PREFIX_EMITTER, &[wormhole_emitter.bump]],
          ],
        ),
        wormhole_config.batch_id,
        payload,
        wormhole_config.finality.try_into().unwrap(),
      )?;
    }

    Ok(())
  }
}