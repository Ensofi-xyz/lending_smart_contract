use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{
    amount::TotalRepayLoanAmountParams,
    common::{ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED},
    states::loan_offer::LoanOfferAccount,
    utils, LoanOfferStatus, RepayOfferError, SystemRepayLoanOfferEvent,
    HOT_WALLET_PUBKEY,
};

#[derive(Accounts)]
#[instruction(loan_offer_id: String)]
pub struct RepayLoanOfferNative<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    #[account(
        constraint = lend_mint_asset.key() == loan_offer.lend_mint_token @ RepayOfferError::InvalidMintAsset,
    )]
    pub lend_mint_asset: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = lend_mint_asset,
        associated_token::authority = borrower
    )]
    pub borrower_ata_lend_asset: Account<'info, TokenAccount>,
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
    pub loan_offer: Account<'info, LoanOfferAccount>,
    #[account(
        mut,
        associated_token::mint = lend_mint_asset,
        associated_token::authority = Pubkey::from_str(HOT_WALLET_PUBKEY).unwrap()
    )]
    pub hot_wallet_ata_lend_asset: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> RepayLoanOfferNative<'info> {
    pub fn repay_loan_offer_native(&mut self) -> Result<()> {
        self.validate_loan_offer()?;

        let total_amount = utils::amount::get_total_repay_loan_amount(TotalRepayLoanAmountParams {
            borrow_amount: self.loan_offer.borrow_amount,
            borrower_fee_percent: self.loan_offer.borrower_fee_percent,
            duration: self.loan_offer.duration,
            interest: self.loan_offer.interest,
        });

        if total_amount > self.borrower_ata_lend_asset.amount {
            return err!(RepayOfferError::NotEnoughAmount);
        }

        self.repay_lend_asset_to_hot_wallet(total_amount)?;

        self.loan_offer
            .sub_lamports(self.loan_offer.collateral_amount)?;
        self.borrower
            .add_lamports(self.loan_offer.collateral_amount)?;
        self.loan_offer.status = LoanOfferStatus::BorrowerPaid;

        self.emit_event_repay_loan_offer(self.loan_offer.collateral_amount)?;

        Ok(())
    }

    fn repay_lend_asset_to_hot_wallet(&mut self, repay_amount: u64) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.borrower_ata_lend_asset.to_account_info(),
            mint: self.lend_mint_asset.to_account_info(),
            to: self.hot_wallet_ata_lend_asset.to_account_info(),
            authority: self.borrower.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

        transfer_checked(cpi_ctx, repay_amount, self.lend_mint_asset.decimals)
    }

    fn validate_loan_offer(&self) -> Result<()> {
        // No need to check timestamp (already check status)
        // let current_timestamp = Clock::get().unwrap().unix_timestamp;
        // let end_borrowed_loan_offer = self.loan_offer.started_at + self.loan_offer.duration as i64;

        // if current_timestamp > end_borrowed_loan_offer {
        //     return err!(LoanOfferError::LoanOfferExpired);
        // }

        Ok(())
    }

    fn emit_event_repay_loan_offer(&mut self, collateral_amount: u64) -> Result<()> {
        emit!(SystemRepayLoanOfferEvent {
            lender: self.loan_offer.lender.key(),
            borrower: self.borrower.key(),
            interest: self.loan_offer.interest,
            loan_amount: self.loan_offer.borrow_amount,
            loan_offer_id: self.loan_offer.offer_id.clone(),
            tier_id: self.loan_offer.tier_id.clone(),
            collateral_amount,
            status: self.loan_offer.status,
        });

        Ok(())
    }
}
