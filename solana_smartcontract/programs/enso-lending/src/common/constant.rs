use anchor_lang::prelude::{borsh, AnchorDeserialize, AnchorSerialize, InitSpace};

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum LendOfferStatus {
    Created,
    Canceling,
    Canceled,
    Loaned,
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum LoanOfferStatus {
    Matched,
    FundTransferred,
    Repay,
    BorrowerPaid,
    Liquidating,
    Liquidated,
    Finished
}

pub const ENSO_SEED: &[u8] = b"enso";
pub const SETTING_ACCOUNT_SEED: &[u8] = b"setting_account";
pub const ASSET_SEED: &[u8] = b"asset";
pub const VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED: &[u8] = b"vault_authority_loan_offer";
pub const LEND_OFFER_ACCOUNT_SEED: &[u8] = b"lend_offer";
pub const LOAN_OFFER_ACCOUNT_SEED: &[u8] = b"loan_offer";
pub const WORMHOLE_MESSAGE_SEED: &[u8] = b"wormhole_message";

#[cfg(feature = "staging")]
pub const OPERATE_SYSTEM_PUBKEY: &str = "sysvYFEXhxW7FP32Ha15BBGWBEfMq1e1ScvFq61u5mG";
#[cfg(not(feature = "staging"))]
pub const OPERATE_SYSTEM_PUBKEY: &str = "opty8HWBKX3wW8c9qMPkmB4xnrCpMWWmQwqq7yGzmr4";
#[cfg(feature = "staging")]
pub const HOT_WALLET_PUBKEY: &str = "hotbEp8jbFUwfAGTUtLupGXE2JtrfZENLgRcSQsYk56";
#[cfg(not(feature = "staging"))]
pub const HOT_WALLET_PUBKEY: &str = "Hot7zcvBTa3NybAnKrKtjcW1yJcoDWao39ZAoBn4mfPu";

pub const SUI_CHAIN_ADDRESS: &str = "0x44f136283e552098e9676c70c91ec5517153e65244b749b979d70fcc7ee15f9e";

#[cfg(feature = "dev")]
pub const MIN_BORROW_HEALTH_RATIO: f64 = 1.1;
#[cfg(not(feature = "dev"))]
pub const MIN_BORROW_HEALTH_RATIO: f64 = 1.2;

pub const DISCRIMINATOR: u8 = 0;

pub const HEX_MIN_WIDTH: u8 = 16;

pub const MAX_ALLOWED_INTEREST: f64 = 200.0;

pub const POSTED_TIMESTAMP_THRESHOLD: u32 = 30 * 60;

pub const CREATE_LOAN_OFFER_CROSS_CHAIN_FUNCTION: &str = "create_loan_offer_cross_chain";
pub const CANCEL_COLLATERAL_FUNCTION: &str = "cancel_collateral";

pub const REFUND_COLLATERAL_CROSS_CHAIN_FUNCTION: &str = "refund_collateral_to_repaid_borrower";

pub const SOL_CHAIN_ID: u16 = 1;
pub const SUI_CHAIN_ID: u16 = 21;
pub const CHAIN_IDS: [u16; 2] = [SOL_CHAIN_ID, SUI_CHAIN_ID]; 