use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{constant::{ENSO_SEED, LEND_OFFER_ACCOUNT_SEED, SETTING_ACCOUNT_SEED}, Asset, CreateLendOfferEvent, LendOfferAccount, LendOfferError, LendOfferStatus, SettingAccount, ASSET_SEED, DISCRIMINATOR, MAX_ALLOWED_INTEREST};

#[derive(Accounts)]
#[instruction(offer_id: String, tier_id: String, interest: f64)]
pub struct CreateLendOffer<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,
    #[account(
        seeds = [
            ENSO_SEED.as_ref(),
            ASSET_SEED.as_ref(),
            mint_asset.key().as_ref(),
            crate::ID.key().as_ref(),
        ],
        bump
    )]
    pub lend_asset: Account<'info, Asset>,
    #[account(
        constraint = mint_asset.key() == lend_asset.token_mint @ LendOfferError::InvalidMintAsset,
    )]
    pub mint_asset: Account<'info, Mint>,
    #[account(
        mut,
        constraint = lender_ata_asset.amount >= setting_account.amount @ LendOfferError::NotEnoughAmount,
        associated_token::mint = mint_asset,
        associated_token::authority = lender
    )]
    pub lender_ata_asset: Account<'info, TokenAccount>,
    #[account(
        seeds = [
            ENSO_SEED.as_ref(), 
            SETTING_ACCOUNT_SEED.as_ref(),
            tier_id.as_bytes(), 
            crate::ID.key().as_ref(), 
        ],
        bump = setting_account.bump
    )]
    pub setting_account: Account<'info, SettingAccount>,
    #[account(
        init,
        payer = lender,
        space = (DISCRIMINATOR as usize) + LendOfferAccount::INIT_SPACE,
        seeds = [
            ENSO_SEED.as_ref(), 
            LEND_OFFER_ACCOUNT_SEED.as_ref(), 
            lender.key().as_ref(), 
            offer_id.as_bytes(),
            crate::ID.key().as_ref(), 
        ],
        bump
    )]
    pub lend_offer: Account<'info, LendOfferAccount>,
    #[account(
        mut,
        associated_token::mint = mint_asset,
        associated_token::authority = setting_account.receiver
    )]
    pub hot_wallet_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateLendOffer<'info> {
    pub fn create_lend_offer(
        &mut self,
        bumps: &CreateLendOfferBumps,
        offer_id: String,
        interest: f64,
    ) -> Result<()> {
            if interest <= (0 as f64) {
                return err!(LendOfferError::InterestGreaterThanZero);
            }

            if interest >= MAX_ALLOWED_INTEREST {
                return err!(LendOfferError::InterestOverLimit);
            }

            let SettingAccount { 
                mut amount, 
                lender_fee_percent, 
                duration, 
                .. 
            } = self.setting_account.clone().into_inner();

            amount = amount * 10_u64.pow(self.mint_asset.decimals as u32);
            
            self.lend_offer.set_inner(LendOfferAccount {
                amount,
                duration,
                bump: bumps.lend_offer,
                interest,
                lender_fee_percent,
                lender: self.lender.key(),
                lend_mint_token: self.mint_asset.key(),
                offer_id: offer_id.clone(),
                status: LendOfferStatus::Created,
            });

            self.deposit(amount)?;
            self.emit_event_create_lend_offer()?;
            
            Ok(())
    }

    fn deposit(&mut self, amount: u64) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(), 
            TransferChecked {
                from: self.lender_ata_asset.to_account_info(),
                mint: self.mint_asset.to_account_info(),
                to: self.hot_wallet_ata.to_account_info(),
                authority: self.lender.to_account_info(),
            }
        );

        transfer_checked(
            ctx,
            amount,
            self.mint_asset.decimals,
        )
    }

    fn emit_event_create_lend_offer(&mut self) -> Result<()> {
        emit!(CreateLendOfferEvent {
            lender: self.lender.key(),
            interest: self.lend_offer.interest,
            lender_fee_percent: self.lend_offer.lender_fee_percent,
            amount: self.lend_offer.amount,
            duration: self.lend_offer.duration,
            offer_id: self.lend_offer.offer_id.clone(),
            tier_id: self.setting_account.tier_id.clone(),
        });
        
        Ok(())
    }
}
