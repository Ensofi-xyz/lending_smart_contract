use anchor_lang::error_code;

#[error_code]
pub enum SettingAccountError {
    #[msg("Invalid tier id")]
    InvalidTierId,   
    #[msg("Invalid owner account")]
    InvalidOwner, 
}

#[error_code]
pub enum LendOfferError {
    #[msg("Invalid Lend Amount")]
    InvalidLendAmount,
    #[msg("Not enough amount")]
    NotEnoughAmount,
    #[msg("Invalid mint asset")]
    InvalidMintAsset,
    #[msg("Interest must be greater than 0")]
    InterestGreaterThanZero,
    #[msg("Lend offer is not initialized or not belong to lender")]
    InvalidLender,
    #[msg("Lend offer status is invalid")]
    InvalidOfferStatus,
    #[msg("Invalid offer id")]
    InvalidOfferId,
    #[msg("Invalid receiver")]
    InvalidReceiver,
    #[msg("Interest over limit")]
    InterestOverLimit,
    #[msg("Invalid signer")]
    InvalidSigner
}

#[error_code]
pub enum LoanOfferError {
    #[msg("Invalid receiver")]
    InvalidReceiver,
    #[msg("Can not deposit collateral to loan offer that not available")]
    CanNotDepositCollateralToContractThatNotAvailable,
    #[msg("Can not take a loan because health ratio is not valid")]
    CanNotTakeALoanBecauseHealthRatioIsNotValid,
    #[msg("Invalid price feed account for collateral asset")]
    InvalidPriceFeedAccountForCollateralAsset,
    #[msg("Invalid price feed account for lend asset")]
    InvalidPriceFeedAccountForLendAsset,
    #[msg("Not enough amount")]
    NotEnoughAmount,
    #[msg("Invalid collateral mint asset")]
    InvalidCollateralMintAsset,
    #[msg("Invalid Lend mint asset")]
    InvalidLendMintAsset,
    #[msg("Loan offer status is invalid")]
    InvalidOfferStatus,
    #[msg("lend offer is not available")]
    LendOfferIsNotAvailable,
    #[msg("Health ratio limit")]
    HealthRatioLimit,
    #[msg("Loan offer expired")]
    LoanOfferExpired,
    #[msg("Loan offer is not expired")]
    LoanOfferNotExpired,
    #[msg("Invalid hot wallet account")]
    InvalidHotWallet,
    #[msg("Invalid operator system account")]
    InvalidSystem,
    #[msg("Invalid borrower")]
    InvalidBorrower,
    #[msg("Invalid loan offer")]
    InvalidLoanOffer,
    #[msg("Invalid borrow amount")]
    InvalidBorrowAmount,
    #[msg("Loan offer not available to withdraw")]
    NotAvailableToWithdraw,
    #[msg("Health ratio invalid")]
    HealthRatioInvalid,
    #[msg("Can not create loan cause lend interest updated")]
    CanNotCreateLoanCauseLendInterestUpdated,
    #[msg("Invalid price feed account")]
    InvalidPriceFeedAccount,
    #[msg("Invalid asset account")]
    InvalidAssetAccount,
    #[msg("Not enough collateral")]
    NotEnoughCollateral,
    #[msg("Invalid initializer vault authority")]
    InvalidInitializerVaultAuthority,
    #[msg("Invalid target function")]
    InvalidTargetFunction,
    #[msg("Lend offer id not match")]
    LendOfferIdNotMatch,
    #[msg("Tier id not match")]
    TierIdNotMatch,
    #[msg("Posted vaa expired")]
    PostedVaaExpired,
    #[msg("Borrower signed loan offer")]
    BorrowerSignedLoanOffer,
    #[msg("Loan offer in validity")]
    LoanOfferInValidity,
    #[msg("Invalid lend offer amount")]
    InvalidLendOfferAmount,
    #[msg("Invalid chain id")]
    InvalidChainId,
    #[msg("Withdraw amount not match")]
    WithdrawAmountNotMatch,
}

#[error_code]
pub enum RepayOfferError {
    #[msg("Invalid mint asset of loan offer")]
    InvalidMintAsset,
    #[msg("Not enough assets")]
    NotEnoughAmount,
    #[msg("Loan offer is not available")]
    LoanOfferIsNotAvailable,
    #[msg("Invalid lend amount")]
    InvalidLendAmount,
    #[msg("Loan offer not belong to borrower")]
    InvalidBorrower,
    #[msg("Invalid collateral amount")]
    InvalidCollateralAmount,
    #[msg("Invalid offer status")]
    InvalidOfferStatus,
    #[msg("Loan offer not belong to lender")]
    InvalidLender,
    #[msg("Invalid repay lender time")]
    TimeUnmetException
}

#[error_code]
pub enum LiquidateOfferError {
    #[msg("Loan offer not belong to lender")]
    InvalidLender,
     #[msg("Loan offer not belong to borrower")]
    InvalidBorrower,
    #[msg("Loan offer status is invalid")]
    InvalidOfferStatus,
    #[msg("Invalid lend amount")]
    InvalidLendAmount,
    #[msg("Not have enough amount of assets")]
    NotEnoughAmount,
    #[msg("Invalid mint asset")]
    InvalidMintAsset,
    #[msg("Invalid operator system account")]
    InvalidSystem,
}

#[error_code]
pub enum WormholeError {
    #[msg("Invalid owner account")]
    InvalidOwner,
    #[msg("Invalid wormhole system account")]
    InvalidSystem,
    #[msg("Invalid foreign emitter")]
    InvalidForeignEmitter,
    #[msg("Invalid message")]
    InvalidMessage,
    #[msg("Not support this chain id")]
    NotSupportThisChainId,
    #[msg("Invalid sequence")]
    InvalidSequence,
    #[msg("Invalid fee collector")]
    InvalidWormholeFeeCollector,
    #[msg("Invalid wormhole config")]
    InvalidWormholeConfig
}

#[error_code]
pub enum EmitterAccountError {
    #[msg("Invalid owner account")]
    InvalidOwner,
}

#[error_code]
pub enum ParseVaaError {
    #[msg("Invalid target chain")]
    InvalidTargetChain,
    #[msg("Invalid collateral amount")]
    InvalidCollateralAmount,
    #[msg("Invalid collateral decimal")]
    InvalidCollateralDecimal,
    #[msg("Invalid foreign emitter")]
    InvalidForeignEmitter,
    #[msg("Invalid Lend amount")]
    InvalidLendAmount,
    #[msg("Invalid remaining collateral amount")]
    InvalidRemainingCollateralAmount
}