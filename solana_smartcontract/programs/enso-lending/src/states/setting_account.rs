pub use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct SettingAccount {
    pub amount: u64,
    pub duration: u64,
    pub owner: Pubkey,
    pub receiver: Pubkey,
    // Note: Not used 2 fields, space for future to used
    pub lend_mint_asset: Pubkey,
    pub collateral_mint_asset: Pubkey,
    #[max_len(50)]
    pub tier_id: String,
    pub lender_fee_percent: f64,
    pub borrower_fee_percent: f64,
    // Note: Not used 2 fields, space for future to used
    pub lend_price_feed: Pubkey,
    pub collateral_price_feed: Pubkey,
    pub bump: u8,
}