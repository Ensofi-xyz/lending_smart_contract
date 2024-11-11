use anchor_lang::prelude::*;
use wormhole_anchor_sdk::wormhole::{self, program::Wormhole};


use crate::{
  common::{ENSO_SEED, LEND_OFFER_ACCOUNT_SEED}, ForeignChain, LendOfferAccount, LendOfferStatus, LoanOfferAccount, LoanOfferError, RequestCancelCollateralCrossChainEvent, WormholeConfig, WormholeEmitter, WormholeError, WormholeMessage, CANCEL_COLLATERAL_FUNCTION, LOAN_OFFER_ACCOUNT_SEED, WORMHOLE_SENT_SEED
};
use crate::utils::vaa;

#[derive(Accounts)]
#[instruction(
  tier_id: String,
  lend_offer_id: String,
  loan_offer_id: String, 
  vaa_hash: [u8; 32],
)]
pub struct RequestCancelLoanedCrossChain<'info> {
  #[account(mut)]
  pub borrower: Signer<'info>,

  pub lender: SystemAccount<'info>,

  #[account(
    mut,
    seeds = [
      ENSO_SEED.as_ref(), 
      LEND_OFFER_ACCOUNT_SEED.as_ref(), 
      lender.key().as_ref(), 
      lend_offer_id.as_bytes(),
      crate::ID.key().as_ref(), 
    ],
    bump = lend_offer.bump
  )]
  pub lend_offer: Box<Account<'info, LendOfferAccount>>,

  #[account(
    mut,
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

  #[account(mut)]
  pub foreign_chain: Account<'info, ForeignChain>,

  #[account(
    seeds = [WormholeConfig::SEED_PREFIX],
    bump,
  )]
  pub config: Account<'info, WormholeConfig>,

  pub wormhole_program: Program<'info, Wormhole>,

  #[account(
    mut,
    address = config.wormhole.bridge @ WormholeError::InvalidWormholeConfig
  )]
  pub wormhole_bridge: Account<'info, wormhole::BridgeData>,

  #[account(
    mut,
    address = config.wormhole.fee_collector @ WormholeError::InvalidWormholeFeeCollector
  )]
  pub wormhole_fee_collector: Account<'info, wormhole::FeeCollector>,

  #[account(
    seeds = [WormholeEmitter::SEED_PREFIX],
    bump,
  )]
  pub wormhole_emitter: Account<'info, WormholeEmitter>,

  #[account(
    mut,
    address = config.wormhole.sequence @ WormholeError::InvalidSequence
  )]
  pub wormhole_sequence: Account<'info, wormhole::SequenceTracker>,

  #[account(
    mut,
    seeds = [
      WORMHOLE_SENT_SEED,
      &wormhole_sequence.next_value().to_le_bytes()[..]
    ],
    bump,
  )]
  /// CHECK: Wormhole Message.
  pub wormhole_message: UncheckedAccount<'info>,

  pub system_program: Program<'info, System>,

  pub clock: Sysvar<'info, Clock>,

	pub rent: Sysvar<'info, Rent>,
}

impl<'info> RequestCancelLoanedCrossChain<'info> {
  pub fn request_cancel_loaned_cross_chain(
    &mut self,
    bumps: &RequestCancelLoanedCrossChainBumps,
    tier_id: String,
    lend_offer_id: String,
    _loan_offer_id: String,
    _vaa_hash: [u8; 32],
  ) -> Result<()> {
    self.validate_posted_vaa()?;

    let posted_vaa = &self.posted.clone().into_inner();
    let WormholeMessage::Message { payload } = posted_vaa.data();
    let ( 
      target_chain,
      chain_address,
      _target_function,
      posted_tier_id,
      posted_lend_offer_id,
      lend_amount,
      _collateral_amount,
      _collateral_address,
      borrower_address
    ) = self.parse_create_loan_payload(&payload).unwrap();

    if self.lend_offer.status != LendOfferStatus::Loaned {
      return err!(LoanOfferError::InvalidOfferStatus);
    }

    let _ = self.verify_payload_message_data(
      posted_lend_offer_id.clone(), 
      lend_amount,
      posted_tier_id,
      tier_id.clone(),
      borrower_address
    )?; 

    let payload_message = self.gen_cancel_loan_payload(
      self.foreign_chain.chain_id.clone(),
      self.foreign_chain.chain_address.clone(),
      CANCEL_COLLATERAL_FUNCTION.to_owned(),
      lend_offer_id.clone(),
      self.borrower.key().to_string(),
    ).unwrap();

    let send_message_fee = self.wormhole_bridge.fee();
		if send_message_fee > 0 {
			let _ = self.transfer_message_fee(send_message_fee);
		}

    let config = &self.config;
    wormhole::post_message(
			CpiContext::new_with_signer(
        self.wormhole_program.to_account_info(),
        wormhole::PostMessage {
          config: self.wormhole_bridge.to_account_info(),
          message: self.wormhole_message.to_account_info(),
          emitter: self.wormhole_emitter.to_account_info(),
          sequence: self.wormhole_sequence.to_account_info(),
          payer: self.borrower.to_account_info(),
          fee_collector: self.wormhole_fee_collector.to_account_info(),
          clock: self.clock.to_account_info(),
          rent: self.rent.to_account_info(),
          system_program: self.system_program.to_account_info(),
        },
        &[
          &[
            WORMHOLE_SENT_SEED.as_ref(),
            &self.wormhole_sequence.next_value().to_le_bytes()[..],
            &[bumps.wormhole_message],
          ],
           &[wormhole::SEED_PREFIX_EMITTER, &[bumps.wormhole_emitter]],
        ],
			),
			config.batch_id,
			payload_message,
			config.finality.try_into().unwrap(),
		)?;

    self.emit_event_request_cancel_loaned_cross_chain(
      lend_offer_id,
      chain_address,
      target_chain
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

  fn parse_create_loan_payload(
    &self,
    posted_vaa: &Vec<u8>,
  ) -> Result<(u16, String, String, String, String, u64, u64, String, String)> {
      Ok(vaa::parse_create_loan_payload(posted_vaa)?)
  }

  fn verify_payload_message_data(
    &self,
    lend_offer_id: String,
    lend_amount: u64,
    posted_tier_id: String,
    tier_id: String,
    borrower_address: String
  ) -> Result<()> {
    if lend_offer_id != self.lend_offer.offer_id {
      return err!(LoanOfferError::LendOfferIdNotMatch);
    }

    if lend_amount != self.lend_offer.amount {
      return err!(LoanOfferError::InvalidLendOfferAmount);
    }

    if posted_tier_id != tier_id {
      return err!(LoanOfferError::TierIdNotMatch);
    }

    if borrower_address != self.borrower.key().to_string() {
      return err!(LoanOfferError::InvalidBorrower);
    }

    if self.loan_offer.borrower.key() == self.borrower.key() {
      return err!(LoanOfferError::BorrowerSignedLoanOffer);
    }

    Ok(())
  }

  fn gen_cancel_loan_payload(
		&self,
		target_chain: u16,
		chain_address: String,
		target_function: String,
		lend_offer_id: String,
    borrower_address: String
	) -> Result<Vec<u8>> {
		let payload = format!("{},{},{},{},{}", target_chain, chain_address, target_function, lend_offer_id, borrower_address);
		Ok(payload.into_bytes())
	}

  fn transfer_message_fee(&self, fee: u64) -> Result<()> {
		Ok(solana_program::program::invoke(
			&solana_program::system_instruction::transfer(
        &self.borrower.key(),
        &self.wormhole_fee_collector.key(),
        fee,
			),
			&self.to_account_infos(),
		)?)
	}

  fn emit_event_request_cancel_loaned_cross_chain(
    &self,
    lend_offer_id: String,
    chain_address: String,
    target_chain: u16
  ) -> Result<()> {
    emit!(RequestCancelCollateralCrossChainEvent {
      borrower: self.borrower.key(),
      lend_offer_id,
      chain_address,
      target_chain
    });

    Ok(())
  }
}