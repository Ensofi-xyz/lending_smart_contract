use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{LoanOfferError, MIN_BORROW_HEALTH_RATIO};

use super::convert_to_usd_price;

pub struct HealthRatioParams<'a> {
    pub collateral_price_feed_account: &'a PriceUpdateV2,
    pub collateral_amount: u64,
    pub collateral_price_feed_id: String,
    pub collateral_max_price_age_seconds: u64,
    pub collateral_decimals: u8,
    pub lend_price_feed_account: &'a PriceUpdateV2,
    pub lend_amount: u64,
    pub lend_price_feed_id: String,
    pub lend_max_price_age_seconds: u64,
    pub lend_decimals: u8,
}

pub fn validate_health_ratio(params: HealthRatioParams) -> Result<()> {
    let (health_ratio, _, _) = get_health_ratio_and_assets_price(params);

    msg!("Health ratio: {}", health_ratio);

    if health_ratio < MIN_BORROW_HEALTH_RATIO {
        return err!(LoanOfferError::HealthRatioInvalid);
    }

    Ok(())
}

pub fn get_health_ratio_and_assets_price(params: HealthRatioParams) -> (f64, f64, f64) {
    let HealthRatioParams {
        collateral_amount,
        collateral_max_price_age_seconds,
        collateral_price_feed_account,
        collateral_price_feed_id,
        lend_amount,
        lend_max_price_age_seconds,
        lend_price_feed_account,
        lend_price_feed_id,
        collateral_decimals,
        lend_decimals,
    } = params;

    let (convert_collateral_amount_to_usd, collateral_price) = convert_to_usd_price(
        collateral_price_feed_account,
        &collateral_price_feed_id,
        collateral_amount as f64 / 10f64.powf(collateral_decimals as f64),
        collateral_max_price_age_seconds,
    )
    .unwrap();
    msg!(
        "Convert collateral amount to USD: {}",
        convert_collateral_amount_to_usd
    );

    let (convert_lend_amount_to_usd, lend_price) = convert_to_usd_price(
        lend_price_feed_account,
        &lend_price_feed_id,
        lend_amount as f64 / 10f64.powf(lend_decimals as f64),
        lend_max_price_age_seconds,
    )
    .unwrap();
    msg!("Convert lend amount to USD: {}", convert_lend_amount_to_usd);

    let health_ratio = convert_collateral_amount_to_usd / convert_lend_amount_to_usd;

    return (health_ratio, collateral_price, lend_price);
}
