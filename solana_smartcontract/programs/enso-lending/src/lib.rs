use anchor_lang::prelude::*;

mod program_id;
use program_id::PROGRAM_ID;
mod contexts;
use contexts::*;
mod states;
use states::*;
mod common;
use common::*;
mod utils;
use utils::*;

declare_id!(PROGRAM_ID);

#[program]
pub mod enso_lending {
    use super::*;

    pub fn init_setting_account(
        ctx: Context<InitSettingAccount>,
        tier_id: String,
        amount: u64,
        duration: u64,
        lender_fee_percent: f64,
        borrower_fee_percent: f64,
    ) -> Result<()> {
        ctx.accounts.init_setting_account(
            &ctx.bumps,
            tier_id.clone(),
            amount,
            duration,
            lender_fee_percent,
            borrower_fee_percent,
        )?;

        Ok(())
    }

    pub fn edit_setting_account(
        ctx: Context<EditSettingAccount>,
        _tier_id: String,
        amount: Option<u64>,
        duration: Option<u64>,
        lender_fee_percent: Option<f64>,
        borrower_fee_percent: Option<f64>,
    ) -> Result<()> {
        ctx.accounts.edit_setting_account(
            amount,
            duration,
            lender_fee_percent,
            borrower_fee_percent,
        )?;

        Ok(())
    }

    pub fn close_setting_account(ctx: Context<CloseSettingAccount>, tier_id: String) -> Result<()> {
        ctx.accounts.close_setting_account(tier_id)?;

        Ok(())
    }

    pub fn init_asset(
        ctx: Context<InitAsset>, 
        name: String,
        is_lend: bool,
        is_collateral: bool,
        price_feed_id: String,
        max_price_age_seconds: u64,
        token_address: Option<String>,
        chain_id: u16
    ) -> Result<()> {
        ctx.accounts.init_asset(
            name, 
            is_lend, 
            is_collateral, 
            price_feed_id, 
            max_price_age_seconds, 
            token_address,
            chain_id,
            &ctx.bumps
        )?;

        Ok(())
    }

    pub fn edit_asset(
        ctx: Context<EditAsset>, 
        name: Option<String>,
        is_lend: Option<bool>,
        is_collateral: Option<bool>,
        price_feed_id: Option<String>,
        max_price_age_seconds: Option<u64>,
        token_address: Option<String>
    ) -> Result<()> {
        ctx.accounts.edit_asset(
            name, 
            is_lend, 
            is_collateral, 
            price_feed_id, 
            max_price_age_seconds,
            token_address
        )?;

        Ok(())
    }

    pub fn init_vault_authority(ctx: Context<InitVaultAuthority>) -> Result<()> {
        ctx.accounts.init_vault_authority(&ctx.bumps)?;
        
        Ok(())
    }

    pub fn create_lend_offer(
        ctx: Context<CreateLendOffer>,
        offer_id: String,
        _tier_id: String,
        interest: f64,
    ) -> Result<()> {
        ctx.accounts.create_lend_offer(&ctx.bumps, offer_id, interest)?;

        Ok(())
    }

    pub fn edit_lend_offer(
        ctx: Context<EditLendOffer>,
        _offer_id: String,
        interest: f64,
    ) -> Result<()> {
        ctx.accounts.edit_lend_offer(interest)?;

        Ok(())
    }

    pub fn system_cancel_lend_offer(
        ctx: Context<SystemCancelLendOffer>,
        _offer_id: String,
        _tier_id: String,
        waiting_interest: u64,
    ) -> Result<()> {
        ctx.accounts.system_cancel_lend_offer(waiting_interest)?;

        Ok(())
    }

    pub fn cancel_lend_offer(ctx: Context<CancelLendOffer>, _offer_id: String) -> Result<()> {
        ctx.accounts.cancel_lend_offer()?;

        Ok(())
    }

    pub fn create_loan_offer(
        ctx: Context<CreateLoanOffer>,
        offer_id: String,
        lend_offer_id: String,
        tier_id: String,
        collateral_amount: u64,
        interest: f64
    ) -> Result<()> {
        ctx.accounts.create_loan_offer(
            offer_id,
            lend_offer_id,
            tier_id,
            collateral_amount,
            interest,
            &ctx.bumps,
        )?;
        Ok(())
    }

    pub fn create_loan_offer_native(
        ctx: Context<CreateLoanOfferNative>,
        offer_id: String,
        lend_offer_id: String,
        tier_id: String,
        collateral_amount: u64,
        interest: f64,
    ) -> Result<()> {
        ctx.accounts.create_loan_offer_native(
            &ctx.bumps,
            offer_id,
            lend_offer_id,
            tier_id,
            collateral_amount,
            interest
        )?;

        Ok(())
    }

    pub fn system_update_loan_offer(
        ctx: Context<SystemUpdateLoanOffer>,
        _offer_id: String,
        _tier_id: String,
        borrow_amount: u64,
    ) -> Result<()> {
        ctx.accounts.system_update_loan_offer(borrow_amount)?;

        Ok(())
    }

    pub fn deposit_collateral_loan_offer_native(
        ctx: Context<DepositCollateralLoanOfferNative>,
        _offer_id: String,
        _tier_id: String,
        amount: u64,
    ) -> Result<()> {
        ctx.accounts.deposit_collateral_loan_offer_native(amount)?;

        Ok(())
    }

    pub fn deposit_collateral_loan_offer(
        ctx: Context<DepositCollateralLoanOffer>,
        _offer_id: String,
        _tier_id: String,
        amount: u64,
    ) -> Result<()> {
        ctx.accounts.deposit_collateral_loan_offer(amount)?;

        Ok(())
    }

    pub fn repay_loan_offer_native(ctx: Context<RepayLoanOfferNative>, _loan_offer_id: String) -> Result<()> {
        ctx.accounts.repay_loan_offer_native()?;

        Ok(())
    }

    pub fn repay_loan_offer(ctx: Context<RepayLoanOffer>, _loan_offer_id: String) -> Result<()> {
        ctx.accounts.repay_loan_offer()?;

        Ok(())
    }

    pub fn withdraw_collateral_loan_offer_native(
        ctx: Context<WithdrawCollateralLoanOfferNative>,
        loan_offer_id: String,
        withdraw_amount: u64,
    ) -> Result<()> {
        ctx.accounts.withdraw_collateral_loan_offer_native(loan_offer_id, withdraw_amount)?;
        
        Ok(())
    }

    pub fn withdraw_collateral_loan_offer(
        ctx: Context<WithdrawCollateralLoanOffer>,
        loan_offer_id: String,
        withdraw_amount: u64,
    ) -> Result<()> {
        ctx.accounts.withdraw_collateral_loan_offer(loan_offer_id, withdraw_amount)?;
        
        Ok(())
    }

    pub fn start_liquidate_loan_offer_native_health(
        ctx: Context<StartLiquidateLoanOfferNativeHealth>,
        _offer_id: String,
    ) -> Result<()> {
        ctx.accounts.start_liquidate_loan_offer_native_health()?;

        Ok(())
    }


    pub fn start_liquidate_loan_offer_health(
        ctx: Context<StartLiquidateLoanOfferHealth>,
        _offer_id: String,
    ) -> Result<()> {
        ctx.accounts.start_liquidate_loan_offer_health()?;

        Ok(())
    }

    pub fn start_liquidate_loan_offer_native_expired(
        ctx: Context<StartLiquidateLoanOfferNativeExpired>,
        _offer_id: String,
    ) -> Result<()> {
        ctx.accounts.start_liquidate_loan_offer_native_expired()?;

        Ok(())
    }

    pub fn start_liquidate_loan_offer_expired(
        ctx: Context<StartLiquidateLoanOfferExpired>,
        _offer_id: String,
    ) -> Result<()> {
        ctx.accounts.start_liquidate_loan_offer_expired()?;

        Ok(())
    }

    pub fn finish_liquidate_contract(
        ctx: Context<SystemLiquidateLoanOffer>,
        _loan_offer_id: String,
        collateral_swapped_amount: u64,
        liquidated_price: u64,
        liquidated_tx: String,
    ) -> Result<()> {
        ctx.accounts.system_liquidate_loan_offer(
              collateral_swapped_amount,
              liquidated_price, 
              liquidated_tx
            )?;

        Ok(())
    }

    pub fn system_finish_loan_offer(
      ctx: Context<SystemFinishLoanOffer>,
      _loan_offer_id: String,
      loan_amount: u64,
      interest_amount: u64,
    ) -> Result<()> {
        ctx.accounts.system_finish_loan_offer(
              loan_amount,
              interest_amount
            )?;

        Ok(())
    }

    pub fn system_revert_status(
      ctx: Context<SystemRevertStatus>,
      _offer_id: String
    ) -> Result<()> {
      ctx.accounts
        .system_revert_status()?;

      Ok(())
    }

    pub fn close_lend_offer(
        ctx: Context<CloseLendOffer>,
        offer_id: String
    ) -> Result<()> {
        ctx.accounts.validate_lend_offer(offer_id)?;

        Ok(())
    }

    pub fn init_foreign_emitter_account(
        ctx: Context<InitForeignEmitter>,
        chain: u16,
        address: String
    ) -> Result<()> {
        ctx.accounts.init_foreign_emitter_account(&ctx.bumps, chain, address)?;

        Ok(())
    }

    pub fn create_loan_offer_cross_chain(
        ctx: Context<CreateLoanOfferCrossChain>,
        tier_id: String,
        loan_offer_id: String,
        lend_offer_id: String,
        vaa_hash: [u8; 32],
    ) -> Result<()> {
        ctx.accounts.create_loan_offer_cross_chain(
            &ctx.bumps,
            tier_id,
            loan_offer_id,
            lend_offer_id,
            vaa_hash
        )?;

        Ok(())
    }

    pub fn init_wormhole_emitter_account(
        ctx: Context<InitWormholeEmitter>,
        chain: u16,
        address: String
    ) -> Result<()> {
        ctx.accounts.init_wormhole_emitter_account(&ctx.bumps, chain, address)?;

        Ok(())
    }

    pub fn cancel_loan_offer_cross_chain(
      ctx: Context<CancelLoanOfferCrossChain>,
      tier_id: String,
      loan_offer_id: String,
      vaa_hash: [u8; 32],
    ) -> Result<()> {
      ctx.accounts.cancel_loan_offer_cross_chain(
        &ctx.bumps,
        tier_id,
        loan_offer_id,
        vaa_hash
      )?;

      Ok(())
    }

    pub fn cancel_loaned_offer_cross_chain(
      ctx: Context<CancelLoanedOfferCrossChain>,
      tier_id: String,
      loan_offer_id: String,
      vaa_hash: [u8; 32],
    ) -> Result<()> {
      ctx.accounts.cancel_loaned_offer_cross_chain(
        &ctx.bumps,
        tier_id,
        loan_offer_id,
        vaa_hash
      )?;

      Ok(())
    }

    pub fn repay_loan_offer_cross_chain(
        ctx: Context<RepayLoanOfferCrossChain>,
        loan_offer_id: String,
    ) -> Result<()> {
        ctx.accounts.repay_loan_offer_cross_chain(
            &ctx.bumps,
            loan_offer_id,
        )?;

        Ok(())
    }
}
