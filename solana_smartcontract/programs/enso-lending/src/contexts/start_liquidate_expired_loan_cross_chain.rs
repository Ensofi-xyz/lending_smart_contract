use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use wormhole_anchor_sdk::wormhole::{ self, program::Wormhole };

use crate::{
    common::constant::{ ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED, OPERATE_SYSTEM_PUBKEY },
    Asset,
    ForeignChain,
    LiquidatingCollateralEvent,
    LoanOfferAccount,
    LoanOfferError,
    LoanOfferStatus,
    WormholeConfig,
    WormholeEmitter,
    WormholeError,
    ASSET_SEED,
    START_LIQUIDATE_EXPIRED_LOAN_CROSS_CHAIN,
    WORMHOLE_SENT_SEED,
};

#[derive(Accounts)]
#[instruction(offer_id: String)]
pub struct StartLiquidateExpiredLoanCrossChain<'info> {
    #[account(
    mut,
    constraint = system.key() == Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() @ LoanOfferError::InvalidSystem
  )]
    pub system: Signer<'info>,

    pub borrower: SystemAccount<'info>,

    #[account(
    constraint = collateral_mint_asset.key() == loan_offer.collateral_mint_token @ LoanOfferError::InvalidCollateralMintAsset,
  )]
    pub collateral_mint_asset: Account<'info, Mint>,

    #[account(
    constraint = collateral_asset.is_collateral == true @ LoanOfferError::InvalidAssetAccount,
    seeds = [
      ENSO_SEED.as_ref(),
      ASSET_SEED.as_ref(),
      collateral_mint_asset.key().as_ref(),
      crate::ID.key().as_ref()
    ],
    bump = collateral_asset.bump
  )]
    pub collateral_asset: Account<'info, Asset>,

    #[account(
    mut,
    seeds = [
      ENSO_SEED.as_ref(),
      LOAN_OFFER_ACCOUNT_SEED.as_ref(),
      borrower.key().as_ref(),
      offer_id.as_bytes(),
      crate::ID.key().as_ref()
    ],
    bump
  )]
    pub loan_offer: Account<'info, LoanOfferAccount>,

    #[account(
    mut,
    constraint = collateral_asset.chain_id == foreign_chain.chain_id @ LoanOfferError::InvalidChainId
  )]
    pub foreign_chain: Account<'info, ForeignChain>,

    #[account(seeds = [WormholeConfig::SEED_PREFIX], bump)]
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

    #[account(seeds = [WormholeEmitter::SEED_PREFIX], bump)]
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

impl<'info> StartLiquidateExpiredLoanCrossChain<'info> {
    pub fn start_liquidate_expired_loan_cross_chain(
        &mut self,
        bumps: &StartLiquidateExpiredLoanCrossChainBumps
    ) -> Result<()> {
        self.validate_expired_loan_offer()?;

        let loan_offer = &mut self.loan_offer;
        if loan_offer.status != LoanOfferStatus::FundTransferred {
            return err!(LoanOfferError::InvalidOfferStatus);
        }

        let current_timestamp = Clock::get().unwrap().unix_timestamp;
        loan_offer.liquidating_at = Some(current_timestamp);
        loan_offer.status = LoanOfferStatus::Liquidating;

        let payload_message = self
            .gen_start_liquidate_expired_loan_cross_chain_payload(
                self.foreign_chain.chain_id,
                self.foreign_chain.chain_address.clone(),
                START_LIQUIDATE_EXPIRED_LOAN_CROSS_CHAIN.to_owned(),
                self.loan_offer.lend_offer_id.clone(),
                self.borrower.key().to_string(),
                self.loan_offer.liquidating_at.unwrap()
            )
            .unwrap();

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
                ]
            ),
            config.batch_id,
            payload_message,
            config.finality.try_into().unwrap()
        )?;

        self.emit_event_start_liquidate_contract()?;

        Ok(())
    }

    fn validate_expired_loan_offer(&self) -> Result<()> {
        let current_timestamp = Clock::get().unwrap().unix_timestamp;
        let end_borrowed_loan_offer =
            self.loan_offer.started_at + (self.loan_offer.duration as i64);

        if current_timestamp < end_borrowed_loan_offer {
            return err!(LoanOfferError::LoanOfferNotExpired);
        }

        Ok(())
    }

    fn gen_start_liquidate_expired_loan_cross_chain_payload(
        &self,
        target_chain: u16,
        chain_address: String,
        target_function: String,
        lend_offer_id: String,
        borrower_address: String,
        liquidating_at: i64
    ) -> Result<Vec<u8>> {
        let payload = format!(
            "{},{},{},{},{},{}",
            target_chain,
            chain_address,
            target_function,
            lend_offer_id,
            borrower_address,
            liquidating_at
        );
        Ok(payload.into_bytes())
    }

    fn transfer_message_fee(&self, fee: u64) -> Result<()> {
        Ok(
            solana_program::program::invoke(
                &solana_program::system_instruction::transfer(
                    &self.borrower.key(),
                    &self.wormhole_fee_collector.key(),
                    fee
                ),
                &self.to_account_infos()
            )?
        )
    }

    fn emit_event_start_liquidate_contract(&self) -> Result<()> {
        emit!(LiquidatingCollateralEvent {
            offer_id: self.loan_offer.offer_id.clone(),
            liquidating_at: self.loan_offer.liquidating_at,
            liquidating_price: self.loan_offer.liquidating_price,
        });

        Ok(())
    }
}
