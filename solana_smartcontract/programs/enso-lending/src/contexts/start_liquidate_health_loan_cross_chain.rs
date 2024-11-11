use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use wormhole_anchor_sdk::wormhole::{ self, program::Wormhole };

use crate::{
    common::constant::{ ASSET_SEED, ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED, OPERATE_SYSTEM_PUBKEY },
    health_ratio::{ self, HealthRatioParams },
    Asset,
    ForeignChain,
    LiquidatingCollateralEvent,
    LoanOfferAccount,
    LoanOfferError,
    LoanOfferStatus,
    WormholeConfig,
    WormholeEmitter,
    WormholeError,
    MIN_BORROW_HEALTH_RATIO,
    START_LIQUIDATE_HEALTH_LOAN_CROSS_CHAIN,
    WORMHOLE_SENT_SEED,
};

#[derive(Accounts)]
#[instruction(loan_offer_id: String)]
pub struct StartLiquidateLoanHealthCrossChain<'info> {
    #[account(
    mut,
    constraint = system.key() == Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() @ LoanOfferError::InvalidSystem
  )]
    pub system: Signer<'info>,

    pub borrower: SystemAccount<'info>,

    #[account(
    mut,
    seeds = [
      ENSO_SEED.as_ref(),
      LOAN_OFFER_ACCOUNT_SEED.as_ref(),
      borrower.key().as_ref(),
      loan_offer_id.as_bytes(),
      crate::ID.key().as_ref()
    ],
    bump
  )]
    pub loan_offer: Account<'info, LoanOfferAccount>,

    #[account(
    constraint = collateral_mint_asset.key() == loan_offer.collateral_mint_token @ LoanOfferError::InvalidCollateralMintAsset,
  )]
    pub collateral_mint_asset: Account<'info, Mint>,

    #[account(
    constraint = lend_mint_asset.key() == loan_offer.lend_mint_token @ LoanOfferError::InvalidLendMintAsset,
  )]
    pub lend_mint_asset: Account<'info, Mint>,

    #[account(
    constraint = lend_asset.is_lend == true @ LoanOfferError::InvalidAssetAccount,
    seeds = [
      ENSO_SEED.as_ref(),
      ASSET_SEED.as_ref(),
      lend_mint_asset.key().as_ref(),
      crate::ID.key().as_ref()
    ],
    bump = lend_asset.bump
  )]
    pub lend_asset: Account<'info, Asset>,

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
    constraint = lend_price_feed_account.key() == lend_asset.price_feed_account @ LoanOfferError::InvalidPriceFeedAccount,
  )]
    pub lend_price_feed_account: Account<'info, PriceUpdateV2>,

    #[account(
    constraint = collateral_price_feed_account.key() == collateral_asset.price_feed_account @ LoanOfferError::InvalidPriceFeedAccount,
  )]
    pub collateral_price_feed_account: Account<'info, PriceUpdateV2>,

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

impl<'info> StartLiquidateLoanHealthCrossChain<'info> {
    pub fn start_liquidate_health_loan_cross_chain(
        &mut self,
        bumps: &StartLiquidateLoanHealthCrossChainBumps
    ) -> Result<()> {
        let loan_offer = &mut self.loan_offer;
        if loan_offer.status != LoanOfferStatus::FundTransferred {
            return err!(LoanOfferError::InvalidOfferStatus);
        }

        let (current_health_ratio, current_collateral_price, _) =
            health_ratio::get_health_ratio_and_assets_price(HealthRatioParams {
                collateral_price_feed_account: &self.collateral_price_feed_account,
                collateral_amount: loan_offer.collateral_amount,
                collateral_price_feed_id: self.collateral_asset.price_feed_id.clone(),
                collateral_max_price_age_seconds: self.collateral_asset.max_price_age_seconds,
                collateral_decimals: self.collateral_asset.decimals,
                lend_price_feed_account: &self.lend_price_feed_account,
                lend_amount: loan_offer.borrow_amount,
                lend_price_feed_id: self.lend_asset.price_feed_id.clone(),
                lend_max_price_age_seconds: self.lend_asset.max_price_age_seconds,
                lend_decimals: self.lend_asset.decimals,
            });

        if current_health_ratio < MIN_BORROW_HEALTH_RATIO {
            loan_offer.liquidating_price = Some(current_collateral_price);
            loan_offer.liquidating_at = Some(Clock::get().unwrap().unix_timestamp);
            loan_offer.status = LoanOfferStatus::Liquidating;

            let payload_message = self
                .gen_start_liquidate_loan_health_cross_chain_payload(
                    self.foreign_chain.chain_id,
                    self.foreign_chain.chain_address.clone(),
                    START_LIQUIDATE_HEALTH_LOAN_CROSS_CHAIN.to_owned(),
                    self.loan_offer.lend_offer_id.clone(),
                    self.borrower.key().to_string(),
                    self.loan_offer.liquidating_price.unwrap(),
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
        } else {
            return err!(LoanOfferError::HealthRatioInvalid);
        }
    }

    fn gen_start_liquidate_loan_health_cross_chain_payload(
        &self,
        target_chain: u16,
        chain_address: String,
        target_function: String,
        lend_offer_id: String,
        borrower_address: String,
        liquidating_price: f64,
        liquidating_at: i64
    ) -> Result<Vec<u8>> {
        let payload = format!(
            "{},{},{},{},{},{},{}",
            target_chain,
            chain_address,
            target_function,
            lend_offer_id,
            borrower_address,
            liquidating_price,
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
