use std::str::FromStr;

use anchor_lang::prelude::*;
use wormhole_anchor_sdk::wormhole::{self, program::Wormhole, SEED_PREFIX_EMITTER, Finality};


use crate::{
  common::{ENSO_SEED, LEND_OFFER_ACCOUNT_SEED}, CancelLoanOfferCrossChainEvent, ForeignEmitter, LendOfferAccount, LendOfferStatus, LoanOfferAccount, LoanOfferError, WormholeEmitter, WormholeMessage, CANCEL_COLLATERAL_FUNCTION, DISCRIMINATOR, LOAN_OFFER_ACCOUNT_SEED, SOL_CHAIN_ID, WORMHOLE_MESSAGE_SEED
};
use crate::utils::vaa;

#[derive(Accounts)]
#[instruction(
  tier_id: String,
  loan_offer_id: String, 
  lend_offer_id: String,
  vaa_hash: [u8; 32],
)]
pub struct CancelLoanedOfferCrossChain<'info> {
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
    init,
    payer = borrower,
    space = (DISCRIMINATOR as usize) + LoanOfferAccount::INIT_SPACE,
    seeds = [
      ENSO_SEED.as_ref(),
      LOAN_OFFER_ACCOUNT_SEED.as_ref(),
      borrower.key().as_ref(),
      loan_offer_id.as_bytes(),
      crate::ID.key().as_ref()
    ],
    bump
    )] 
  pub loan_offer:Box<Account<'info, LoanOfferAccount>>,
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
  pub foreign_emitter: Account<'info, ForeignEmitter>,
  #[account(mut)]
  pub wormhole_bridge: Account<'info, wormhole::BridgeData>,
  #[account(mut)]
  pub wormhole_fee_collector: Account<'info, wormhole::FeeCollector>,
  #[account(mut)]
  pub wormhole_sequence: Account<'info, wormhole::SequenceTracker>,
  #[account(
    seeds = [
      ENSO_SEED.as_ref(), 
      SEED_PREFIX_EMITTER.as_ref(),
      &SOL_CHAIN_ID.to_be_bytes(),
      crate::ID.key().as_ref(), 
    ],
    bump,
  )]
  pub wormhole_emitter: Account<'info, WormholeEmitter>,
  #[account(
	mut,
	seeds = [
		ENSO_SEED.as_ref(),
		WORMHOLE_MESSAGE_SEED.as_ref(),
		&wormhole_sequence.next_value().to_le_bytes()[..]
	],
	bump,
	)]
  /// CHECK: initialized and written to by wormhole core bridge
	pub wormhole_message: UncheckedAccount<'info>,
  pub wormhole_program: Program<'info, Wormhole>,
  pub system_program: Program<'info, System>,
  pub clock: Sysvar<'info, Clock>,
	pub rent: Sysvar<'info, Rent>,
}

impl<'info> CancelLoanedOfferCrossChain<'info> {
  pub fn cancel_loaned_offer_cross_chain(
    &mut self,
    bumps: &CancelLoanedOfferCrossChainBumps,
    tier_id: String,
    loan_offer_id: String,
    _vaa_hash: [u8; 32],
  ) -> Result<()> {
    self.validate_posted_vaa()?;

    let posted_vaa = &self.posted.clone().into_inner();
    let WormholeMessage::Message { payload } = posted_vaa.data();
    let ( 
      target_chain,
      target_address,
      _target_function,
      posted_tier_id,
      offer_id,
      _collateral_amount,
      _collateral_address,
      _collateral_token_decimal,
      _collateral_token_symbol,
      borrower_address
    ) = self.parse_create_loan_payload(&payload).unwrap();

    if self.lend_offer.status != LendOfferStatus::Loaned {
      return err!(LoanOfferError::InvalidOfferStatus);
    }

    let _ = self.verify_payload_message_data(
      offer_id.clone(), 
      posted_tier_id,
      tier_id.clone(),
      borrower_address
    )?; 

    let payload_message = self.gen_cancel_loan_payload(
      target_chain,
      target_address.clone(),
      CANCEL_COLLATERAL_FUNCTION.to_owned(),
      loan_offer_id.clone(),
      self.borrower.key().to_string(),
    ).unwrap();

    let send_message_fee = self.wormhole_bridge.fee();
		if send_message_fee > 0 {
			let _ = self.transfer_message_fee(send_message_fee);
		}

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
            ENSO_SEED.as_ref(),
            WORMHOLE_MESSAGE_SEED.as_ref(),
            &self.wormhole_sequence.next_value().to_le_bytes()[..],
            &[bumps.wormhole_message],
          ],
          &[
            ENSO_SEED.as_ref(), 
            SEED_PREFIX_EMITTER.as_ref(),
            &SOL_CHAIN_ID.to_be_bytes(),
            crate::ID.key().as_ref(), 
            &[bumps.wormhole_emitter],
          ],
        ],
			),
			0, //batch_id nonce
			payload_message,
			Finality::Finalized,
		)?;

    self.emit_event_cancel_loaned_offer_cross_chain(
      offer_id,
      loan_offer_id,
      target_address,
      target_chain
    )?;

    Ok(())
  }

  fn validate_posted_vaa(&self) -> Result<()> {
    let posted_emitter_chain = self.posted.meta.emitter_chain;
    let foreign_emitter_chain = self.foreign_emitter.chain;
    let posted_emitter_address = self.posted.meta.emitter_address;
    let foreign_emitter_address = self.foreign_emitter.address.clone();

    Ok(vaa::validate_posted_vaa(
      posted_emitter_chain,
      foreign_emitter_chain,
      posted_emitter_address,
      foreign_emitter_address,
    )?)
  }

  fn parse_create_loan_payload(
    &self,
    posted_vaa: &Vec<u8>,
  ) -> Result<(u16 ,String, String, String, String ,u64, String, u8, String, String)> {
      Ok(vaa::parse_create_loan_payload(posted_vaa)?)
  }

  fn verify_payload_message_data(
    &self,
    lend_offer_id: String,
    posted_tier_id: String,
    tier_id: String,
    borrower_address: String
  ) -> Result<()> {
    if lend_offer_id != self.lend_offer.offer_id {
      return err!(LoanOfferError::LendOfferIdNotMatch);
    }

    if posted_tier_id != tier_id {
      return err!(LoanOfferError::TierIdNotMatch);
    }

    if Pubkey::from_str(&borrower_address).unwrap() != self.borrower.key() {
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
		target_address: String,
		target_function: String,
		loan_offer_id: String,
    borrower_address: String
	) -> Result<Vec<u8>> {
		let payload = format!("{},{},{},{},{}", target_chain, target_address, target_function, loan_offer_id, borrower_address);
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

  fn emit_event_cancel_loaned_offer_cross_chain(
    &self,
    lend_offer_id: String,
    loan_offer_id: String,
    target_address: String,
    target_chain: u16
  ) -> Result<()> {
    emit!(CancelLoanOfferCrossChainEvent {
      borrower: self.borrower.key(),
      lend_offer_id,
      loan_offer_id,
      target_address,
      target_chain
    });

    Ok(())
  }
}