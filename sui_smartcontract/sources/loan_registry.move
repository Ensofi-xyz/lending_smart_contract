module enso_lending::loan_registry {
    use sui::{
        balance::{Self, Balance},
        coin::Coin,
        clock::Clock,
        event,
    };
    use std::string::String;
    use pyth::price_info::PriceInfoObject;
    use enso_lending::{
        offer_registry::Offer,
        asset::Asset,
        utils::{Self, default_rate_factor},
    };

    use fun enso_lending::price_feed::get_value_by_usd as PriceInfoObject.get_value_by_usd;
    use fun std::string::utf8 as vector.to_string;

    const MATCHED_STATUS: vector<u8> = b"Matched";
    const FUND_TRANSFERRED_STATUS: vector<u8> = b"FundTransferred";
    const BORROWER_PAID_STATUS: vector<u8> = b"BorrowerPaid";
    const LIQUIDATING_STATUS: vector<u8> = b"Liquidating";
    const LIQUIDATED_STATUS: vector<u8> = b"Liquidated";
    const FINISHED_STATUS: vector<u8> = b"Finished";

    public struct Liquidation<phantom T1, phantom T2> has store, drop {
        liquidating_at: u64,
        liquidating_price: u64,
        liquidated_tx: Option<String>,
        liquidated_price: Option<u64>,
    }

    public struct LoanKey<phantom T1, phantom T2> has store, copy, drop {
        loan_id: ID,
    }

    public struct Loan<phantom T1, phantom T2> has key, store {
        id: UID,
        offer_id: ID,
        interest: u64,
        amount: u64,
        // === seconds ===
        duration: u64,
        collateral: Balance<T2>,
        lender: address,
        borrower: address,
        start_timestamp: u64,
        liquidation: Option<Liquidation<T1,T2>>,
        // === move to hot wallet ===
        repay_balance: Balance<T1>,
        status: String,
    }

    public struct RequestLoanEvent has copy, drop {
        loan_id: ID,
        offer_id: ID,
        amount: u64,
        duration: u64,
        collateral_amount: u64, 
        lend_token: String,
        collateral_token: String,
        lender: address,
        borrower: address,
        start_timestamp: u64,
    }

    public struct FundTransferredEvent has copy, drop {
        loan_id: ID,
        offer_id: ID,
        amount: u64,
        duration: u64,
        lend_token: String,
        collateral_token: String,
        lender: address,
        borrower: address,
    }

    public struct BorrowerPaidEvent has copy, drop {
        loan_id: ID,
        repay_amount: u64,
        collateral_amount: u64, 
        lend_token: String,
        collateral_token: String,
        borrower: address,
    }

    public struct FinishedLoanEvent has copy, drop {
        loan_id: ID,
        offer_id: ID,
        repay_to_lender_amount: u64,
        lender: address,
        borrower: address,
    }

    public struct WithdrawCollateralEvent has copy, drop {
        loan_id: ID,
        borrower: address,
        withdraw_amount: u64,
        remaining_collateral_amount: u64,
        timestamp: u64,
    }

    public struct DepositCollateralEvent has copy, drop {
        tier_id: ID,
        lend_offer_id: ID,
        interest: u64,
        borrow_amount: u64,
        lender_fee_percent: u64,
        duration: u64,
        lend_token: String,
        lender: address,
        loan_offer_id: ID,
        borrower: address,
        collateral_token: String,
        collateral_amount: u64,
        status: String,
        borrower_fee_percent: u64,
        timestamp: u64,
    }
    
    public struct LiquidatingCollateralEvent has copy, drop {
        loan_id: ID,
        liquidating_price: u64,
        liquidating_at: u64,
    }

    public struct LiquidatedCollateralEvent has copy, drop {
        lender: address,
        borrower: address,
        loan_id: ID,
        collateral_swapped_amount: u64,
        status: String,
        liquidated_price: u64,
        liquidated_tx: String,
        remaining_fund_to_borrower: u64,
    }

    public(package) fun new_loan<LendCoinType, CollateralCoinType>(
        collateral: Coin<CollateralCoinType>,
        offer: &Offer<LendCoinType>,
        borrower: address,
        start_timestamp: u64,
        ctx: &mut TxContext,
    ): Loan<LendCoinType, CollateralCoinType> {
        let collateral_amount = collateral.value<CollateralCoinType>();
        let loan = Loan<LendCoinType, CollateralCoinType> {
            id: object::new(ctx),
            offer_id: object::id(offer),
            interest: offer.interest<LendCoinType>(),
            amount: offer.amount<LendCoinType>(),
            duration: offer.duration<LendCoinType>(),
            collateral: collateral.into_balance<CollateralCoinType>(),
            lender: offer.lender<LendCoinType>(),
            borrower,
            start_timestamp,
            liquidation: option::none<Liquidation<LendCoinType, CollateralCoinType>>(),
            repay_balance: balance::zero<LendCoinType>(),
            status: MATCHED_STATUS.to_string(),
        };

        event::emit(RequestLoanEvent {
            loan_id: object::id(&loan),
            offer_id: object::id(offer),
            amount: offer.amount<LendCoinType>(),
            duration: offer.duration<LendCoinType>(),
            collateral_amount,
            lend_token: utils::get_type<LendCoinType>(),
            collateral_token: utils::get_type<CollateralCoinType>(),
            lender: offer.lender<LendCoinType>(),
            borrower,
            start_timestamp,
        });

        loan
    }

    public(package) fun repay<LendCoinType, CollateralCoinType>(
        loan: &mut Loan<LendCoinType, CollateralCoinType>,
        repay_amount: u64,
        collateral_amount: u64,
        lend_token: String,
        collateral_token: String,
    ) {
        loan.status = BORROWER_PAID_STATUS.to_string();

        event::emit(BorrowerPaidEvent {
            loan_id: object::id(loan),
            repay_amount,
            collateral_amount,
            lend_token,
            collateral_token,
            borrower: loan.borrower(),
        });
    }

    #[allow(lint(self_transfer))]
    public(package) fun withdraw_collateral<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>,
        withdraw_amount: u64,
        remaining_collateral_amount: u64,
        current_timestamp: u64,
    ) {
        event::emit(WithdrawCollateralEvent {
            loan_id: object::id(loan),
            borrower: loan.borrower,
            withdraw_amount,
            remaining_collateral_amount,
            timestamp: current_timestamp,
        });
    }

    public(package) fun deposit_collateral<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>,
        asset_tier: ID,
        lender_fee_percent: u64,
        borrower_fee_percent: u64,
        lend_token: String,
        collateral_token: String,
        total_collateral_amount: u64,
        current_timestamp: u64,
    ) {
        event::emit(DepositCollateralEvent {
            tier_id: asset_tier,
            lend_offer_id: loan.offer_id,
            interest: loan.interest,
            borrow_amount: loan.amount,
            lender_fee_percent,
            duration: loan.duration,
            lend_token,
            lender: loan.lender,
            loan_offer_id: object::id(loan),
            borrower: loan.borrower,
            collateral_token,
            collateral_amount: total_collateral_amount,
            status: loan.status,
            borrower_fee_percent,
            timestamp: current_timestamp,
            }
        );
    }

    public(package) fun system_fund_transfer<LendCoinType, CollateralCoinType>(
        loan: &mut Loan<LendCoinType, CollateralCoinType>,
        lend_token: String,
        collateral_token: String,
        _ctx: &mut TxContext,
    ) {
        loan.status = FUND_TRANSFERRED_STATUS.to_string();

        event::emit(FundTransferredEvent {
            loan_id: object::id(loan),
            offer_id: loan.offer_id,
            amount: loan.amount,
            duration: loan.duration,
            lend_token,
            collateral_token,
            lender: loan.lender,
            borrower: loan.borrower,
        });
    }

    public(package) fun system_finish_loan<LendCoinType, CollateralCoinType>(
        loan: &mut Loan<LendCoinType, CollateralCoinType>,
        repay_to_lender_amount: u64,
    ) {
        loan.status = FINISHED_STATUS.to_string();

        event::emit(FinishedLoanEvent {
            loan_id: object::id(loan),
            offer_id: loan.offer_id,
            repay_to_lender_amount,
            lender: loan.lender,
            borrower: loan.borrower,
        });
    }

    public(package) fun start_liquidate_loan_offer<LendCoinType, CollateralCoinType>(
        loan: &mut Loan<LendCoinType, CollateralCoinType>,
        liquidating_price: u64,
        liquidating_at: u64,
    ) {
        if (loan.liquidation.is_none()) {
            loan.liquidation = option::some<Liquidation<LendCoinType, CollateralCoinType>>(Liquidation {
                liquidating_at,
                liquidating_price,
                liquidated_tx: option::none<String>(),
                liquidated_price: option::none<u64>(),
            });
        } else {
            let liquidation = loan.liquidation.borrow_mut();
            liquidation.liquidating_at = liquidating_at;
            liquidation.liquidating_price = liquidating_price;
        };

        loan.status = LIQUIDATING_STATUS.to_string();

        event::emit(LiquidatingCollateralEvent {
            loan_id: object::id(loan),
            liquidating_price,
            liquidating_at,
        });
    }

    public(package) fun finish_liquidate_loan_offer<LendCoinType, CollateralCoinType>(
        loan: &mut Loan<LendCoinType, CollateralCoinType>,
        collateral_swapped_amount: u64,
        refund_to_borrower_amount: u64,
        liquidated_price: u64,
        liquidated_tx: String,
    ) {
        let liquidation = loan.liquidation.borrow_mut();
        liquidation.liquidated_price = option::some<u64>(liquidated_price);
        liquidation.liquidated_tx = option::some<String>(liquidated_tx);

        loan.status = LIQUIDATED_STATUS.to_string();

        event::emit(LiquidatedCollateralEvent {
            lender: loan.lender,
            borrower: loan.borrower,
            loan_id: object::id(loan),
            collateral_swapped_amount,
            status: loan.status,
            liquidated_price,
            liquidated_tx,
            remaining_fund_to_borrower: refund_to_borrower_amount,
        });
    }

    public(package) fun add_collateral_balance<LendCoinType, CollateralCoinType>(
        loan: &mut Loan<LendCoinType, CollateralCoinType>,
        amount: Balance<CollateralCoinType>,
    ) {
        loan.collateral.join<CollateralCoinType>(amount);
    }

    public(package) fun sub_collateral_balance<LendCoinType, CollateralCoinType>(
        loan: &mut Loan<LendCoinType, CollateralCoinType>,
        amount: u64,
    ): Balance<CollateralCoinType> {
        loan.collateral.split<CollateralCoinType>(amount)
    }

    public fun new_loan_key<LendCoinType, CollateralCoinType>(
        loan_id: ID,
    ): LoanKey<LendCoinType, CollateralCoinType> {
        LoanKey<LendCoinType, CollateralCoinType> {
            loan_id
        }
    }

    public fun is_valid_collateral_amount<LendCoinType, CollateralCoinType>(
        min_health_ratio: u64,
        lend_amount: u64,
        collateral_amount: u64,
        lend_asset: &Asset<LendCoinType>,
        collateral_asset: &Asset<CollateralCoinType>,
        price_info_object_lending: &PriceInfoObject,
        price_info_object_collateral: &PriceInfoObject,
        clock: &Clock,
    ): bool {
        let lend_decimals = lend_asset.decimals<LendCoinType>();
        let collateral_decimals = collateral_asset.decimals<CollateralCoinType>();
        let max_decimals: u64;

        if (lend_decimals > collateral_decimals) {
            max_decimals = (lend_decimals as u64);
        } else  {
            max_decimals = (collateral_decimals as u64);
        };

        let collateral_value_by_usd = price_info_object_collateral.get_value_by_usd<CollateralCoinType>(
            max_decimals, 
            collateral_amount, 
            collateral_asset, 
            clock
        );
        let lend_value_by_usd = price_info_object_lending.get_value_by_usd<LendCoinType>(
            max_decimals, 
            lend_amount, 
            lend_asset, 
            clock
        );
        let current_health_ratio = (collateral_value_by_usd * (default_rate_factor() as u128)) / lend_value_by_usd;

        current_health_ratio >= (min_health_ratio as u128)
    }

    public fun is_expired_loan_offer<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>,
        current_timestamp: u64,
    ): bool {
        let end_borrowed_loan =  loan.start_timestamp + loan.duration;
        current_timestamp >= end_borrowed_loan
    }

    public fun is_matched_status<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>
    ): bool {
        loan.status == MATCHED_STATUS.to_string()
    }

    public fun is_fund_transferred_status<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>
    ): bool {
        loan.status == FUND_TRANSFERRED_STATUS.to_string()
    }

    public fun is_borrower_paid_status<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>
    ): bool {
        loan.status == BORROWER_PAID_STATUS.to_string()
    }

    public fun is_liquidating_status<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>
    ): bool {
        loan.status == LIQUIDATING_STATUS.to_string()
    }

    public fun is_liquidated_status<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>
    ): bool {
        loan.status == LIQUIDATED_STATUS.to_string()
    }

    public fun offer_id<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>
    ): ID {
        loan.offer_id
    }

    public fun amount<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>
    ): u64 {
        loan.amount
    }

    public fun interest<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>
    ): u64 {
        loan.interest
    }

    public fun duration<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>
    ): u64 {
        loan.duration
    }

    public fun borrower<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>
    ): address {
        loan.borrower
    }

    public fun lender<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>
    ): address {
        loan.lender
    }

    public fun collateral_amount<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>,
    ): u64 {
        loan.collateral.value<CollateralCoinType>()
    }

    public fun start_timestamp<LendCoinType, CollateralCoinType>(
        loan: &Loan<LendCoinType, CollateralCoinType>,
    ): u64 {
        loan.start_timestamp
    }
}