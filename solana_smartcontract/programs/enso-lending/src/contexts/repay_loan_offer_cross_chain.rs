use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};
use wormhole_anchor_sdk::wormhole::{self, program::Wormhole, SEED_PREFIX_EMITTER, Finality};

use crate::{
	amount::TotalRepayLoanAmountParams, common::{
    ENSO_SEED,
    LOAN_OFFER_ACCOUNT_SEED, 
		WORMHOLE_MESSAGE_SEED,
  }, foreign_chain, utils, Asset, LoanOfferAccount, LoanOfferError, LoanOfferStatus, RepayOfferError, SystemRepayLoanOfferEvent, WormholeEmitter, HOT_WALLET_PUBKEY, REFUND_COLLATERAL_CROSS_CHAIN_FUNCTION, SOL_CHAIN_ID
};

#[derive(Accounts)]
#[instruction(
	loan_offer_id: String,
	target_chain: u16,
	target_address: String,
	target_function: String,
)]
pub struct RepayLoanOfferCrossChain<'info> {
	#[account(mut)]
  pub borrower: Signer<'info>,
	#[account(
    constraint = collateral_mint_asset.key() == loan_offer.collateral_mint_token @ LoanOfferError::InvalidCollateralMintAsset,
  )]
  pub collateral_mint_asset: Box<Account<'info, Mint>>,
  #[account(
    constraint = lend_mint_asset.key() == loan_offer.lend_mint_token @ LoanOfferError::InvalidLendMintAsset,
  )]
  pub lend_mint_asset: Box<Account<'info, Mint>>,
	#[account(
    constraint = collateral_asset.token_mint == loan_offer.collateral_mint_token @ LoanOfferError::InvalidCollateralMintAsset,
  )]
  pub collateral_asset: Box<Account<'info, Asset>>,
  #[account(
    constraint = lend_mint_asset.key() == loan_offer.lend_mint_token @ LoanOfferError::InvalidLendMintAsset,
  )]
  pub lend_asset: Box<Account<'info, Asset>>,
	#[account(
    mut,
    associated_token::mint = lend_mint_asset,
    associated_token::authority = borrower
  )]
  pub borrower_ata_lend_asset: Box<Account<'info, TokenAccount>>,
	#[account(
    mut,
    constraint = loan_offer.status == LoanOfferStatus::FundTransferred @ RepayOfferError::LoanOfferIsNotAvailable,
    seeds = [
      ENSO_SEED.as_ref(),
      LOAN_OFFER_ACCOUNT_SEED.as_ref(),
      borrower.key().as_ref(),
      loan_offer_id.as_bytes(),
      crate::ID.key().as_ref()
    ],
    bump
  )]
  pub loan_offer: Box<Account<'info, LoanOfferAccount>>,
  #[account(
    mut,
    associated_token::mint = lend_mint_asset,
    associated_token::authority = Pubkey::from_str(HOT_WALLET_PUBKEY).unwrap()
  )]
  pub hot_wallet_ata_lend_asset: Account<'info, TokenAccount>,
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
	pub token_program: Program<'info, Token>,
	pub system_program: Program<'info, System>,
	pub clock: Sysvar<'info, Clock>,
	pub rent: Sysvar<'info, Rent>,
}

impl<'info> RepayLoanOfferCrossChain<'info> {
	pub fn repay_loan_offer_cross_chain(
		&mut self,
		bumps: &RepayLoanOfferCrossChainBumps,
		_loan_offer_id: String,
	) -> Result<()> {
		let target_chain = self.collateral_asset.chain_id;
		let target_address = foreign_chain::get_chain_address_by_chain_id(target_chain).unwrap();
		
		let send_message_fee = self.wormhole_bridge.fee();
		if send_message_fee > 0 {
			let _ = self.transfer_message_fee(send_message_fee);
		}

		let total_amount = utils::amount::get_total_repay_loan_amount(TotalRepayLoanAmountParams {
      borrow_amount: self.loan_offer.borrow_amount,
      borrower_fee_percent: self.loan_offer.borrower_fee_percent,
      duration: self.loan_offer.duration,
      interest: self.loan_offer.interest,
  	});
    
    if total_amount > self.borrower_ata_lend_asset.amount {
      return err!(RepayOfferError::NotEnoughAmount);
    };
		self.repay_lend_asset_to_hot_wallet(total_amount)?;
		self.loan_offer.status = LoanOfferStatus::BorrowerPaid;

		let payload = self.gen_repay_loan_payload(
			target_chain,
			target_address,
			REFUND_COLLATERAL_CROSS_CHAIN_FUNCTION.to_owned(),
			self.loan_offer.lend_offer_id.clone(),
			self.borrower.key().to_string(),
		).unwrap();

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
									&[self.wormhole_emitter.bump]
							],
					],
			),
			0, //batch_id nonce
			payload,
			Finality::Finalized,
		)?;

		let _ = self.emit_event_repay_loan_offer();

		Ok(())
	}

	fn transfer_message_fee(&self, fee: u64) -> Result<()> {
		
		Ok(
			solana_program::program::invoke(
		&solana_program::system_instruction::transfer(
					&self.borrower.key(),
					&self.wormhole_fee_collector.key(),
					fee,
				),
				&self.to_account_infos(),
			)?
		)
	}

	fn repay_lend_asset_to_hot_wallet(&self, repay_amount: u64) -> Result<()> {
    let cpi_ctx = CpiContext::new(
      self.token_program.to_account_info(), 
      TransferChecked {
        from: self.borrower_ata_lend_asset.to_account_info(),
        mint: self.lend_mint_asset.to_account_info(),
        to: self.hot_wallet_ata_lend_asset.to_account_info(),
        authority: self.borrower.to_account_info(),
      }
    );

    transfer_checked(
      cpi_ctx,
      repay_amount,
      self.lend_mint_asset.decimals,
    )
  }

	fn gen_repay_loan_payload(
		&self,
		target_chain: u16,
		target_address: String,
		target_function: String,
		lend_offer_id: String,
		borrower_address: String,
	) -> Result<Vec<u8>> {
		let payload = format!("{},{},{},{},{}", target_chain, target_address, target_function, lend_offer_id, borrower_address);
		Ok(payload.into_bytes())
	}

	fn emit_event_repay_loan_offer(&mut self) -> Result<()> {
    emit!(SystemRepayLoanOfferEvent {
      lender: self.loan_offer.lender.key(),
      borrower: self.borrower.key(),
      interest: self.loan_offer.interest,
      loan_amount: self.loan_offer.borrow_amount,
      loan_offer_id: self.loan_offer.offer_id.clone(),
      tier_id: self.loan_offer.tier_id.clone(),
      collateral_amount: self.loan_offer.collateral_amount,
      status: self.loan_offer.status,
    });
          
    Ok(())
  }
}