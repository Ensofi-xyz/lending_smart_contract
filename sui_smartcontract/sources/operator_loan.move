module enso_lending::operator_loan {
	use std::string::String;
    use sui::coin::Coin;
    use sui::balance::Balance;
    use sui::clock::Clock;

    use pyth::price_info::PriceInfoObject;    
    use enso_lending::{
        version::{Version},
        configuration::Configuration,
        custodian::Custodian,
        state::State,
        loan_registry::{Self, Loan, LoanKey, is_valid_collateral_amount},
        asset::Asset,
        operator::OperatorCap,
        utils::{Self, default_rate_factor, seconds_in_year},
    };

    use fun enso_lending::price_feed::is_valid_price_info_object as PriceInfoObject.is_valid;
    use fun enso_lending::price_feed::get_price as PriceInfoObject.get_price;
    use fun sui::coin::from_balance as Balance.to_coin;

	const EInvalidLoanStatus: u64 = 1;
    const ELoanNotFound: u64 = 2;
    const EPriceInfoObjectLendingIsInvalid: u64 = 3;
    const EPriceInfoObjectCollateralIsInvalid: u64 = 4;
    const ELendCoinIsInvalid: u64 = 5;
    const ECollateralCoinIsInvalid: u64 = 6;
	const ENotEnoughBalanceToRepay: u64 = 7;
	const ECanNotLiquidateValidCollateral: u64 = 8;
    const ECanNotLiquidateUnexpiredLoan: u64 = 9;
    const EInvalidCoinInput: u64 = 11;

    public entry fun system_fund_transfer<LendCoinType, CollateralCoinType>(
        _: &OperatorCap,
        version: &Version,
        state: &mut State,
        loan_id: ID,
        lend_coin: Coin<LendCoinType>,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();
        
        let lend_token = utils::get_type<LendCoinType>();
        let collateral_token = utils::get_type<CollateralCoinType>();

        let loan_key = loan_registry::new_loan_key<LendCoinType, CollateralCoinType>(loan_id);
        assert!(state.contain<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key), ELoanNotFound);
        let loan = state.borrow_mut<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key);

		assert!(loan.is_matched_status<LendCoinType, CollateralCoinType>(), EInvalidLoanStatus);
        transfer::public_transfer(lend_coin, loan.borrower<LendCoinType, CollateralCoinType>());

        loan.system_fund_transfer<LendCoinType, CollateralCoinType>(
            lend_token,
            collateral_token,
            ctx,
        );
    }

    public entry fun system_finish_loan<LendCoinType, CollateralCoinType>(
        _: &OperatorCap,
        version: &Version,
        configuration: &Configuration,
        custodian: &mut Custodian<LendCoinType>,
        state: &mut State, 
        loan_id: ID,
        repay_coin: Coin<LendCoinType>,
        waiting_interest: Coin<LendCoinType>,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();

        let loan_key = loan_registry::new_loan_key<LendCoinType, CollateralCoinType>(loan_id);
        assert!(state.contain<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key), ELoanNotFound);
        let loan = state.borrow_mut<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key);

		assert!(loan.is_borrower_paid_status<LendCoinType, CollateralCoinType>() || loan.is_liquidated_status<LendCoinType, CollateralCoinType>(), EInvalidLoanStatus);
        let interest_amount = ((loan.amount() * loan.interest() / default_rate_factor() * loan.duration() as u128) / (seconds_in_year() as u128) as u64);
        let lender_fee_amount = ((interest_amount * configuration.lender_fee_percent() as u128) / (default_rate_factor() as u128) as u64);
        let repay_to_lender_amount = loan.amount() + interest_amount - lender_fee_amount;

        assert!(repay_coin.value<LendCoinType>() == loan.amount() + interest_amount, ENotEnoughBalanceToRepay);

        let mut repay_balance = repay_coin.into_balance<LendCoinType>();
        let lender_fee_balance = repay_balance.split(lender_fee_amount);
        custodian.add_treasury_balance<LendCoinType>(lender_fee_balance);

        let waiting_interest_balance = waiting_interest.into_balance<LendCoinType>();
        repay_balance.join<LendCoinType>(waiting_interest_balance);

        transfer::public_transfer(repay_balance.to_coin(ctx), loan.lender());
        
        loan.system_finish_loan<LendCoinType, CollateralCoinType>(repay_to_lender_amount);
    }

	public entry fun start_liquidate_loan_offer_health<LendCoinType, CollateralCoinType>(
        _: &OperatorCap,
        version: &Version,
        configuration: &Configuration,
        state: &mut State,
        loan_id: ID,
        price_info_object_lending: &PriceInfoObject,
        price_info_object_collateral: &PriceInfoObject,
        clock: &Clock,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();
		let current_timestamp = clock.timestamp_ms();

        let lend_asset = configuration.borrow<String, Asset<LendCoinType>>(utils::get_type<LendCoinType>());
        let collateral_asset = configuration.borrow<String, Asset<CollateralCoinType>>(utils::get_type<CollateralCoinType>());

        assert!(lend_asset.is_lend_coin<LendCoinType>(), ELendCoinIsInvalid);
        assert!(collateral_asset.is_collateral_coin<CollateralCoinType>(), ECollateralCoinIsInvalid);
        assert!(price_info_object_lending.is_valid<LendCoinType>(lend_asset), EPriceInfoObjectLendingIsInvalid);
        assert!(price_info_object_collateral.is_valid<CollateralCoinType>(collateral_asset), EPriceInfoObjectCollateralIsInvalid);

        let loan_key = loan_registry::new_loan_key<LendCoinType, CollateralCoinType>(loan_id);
        assert!(state.contain<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key), ELoanNotFound);
        let loan = state.borrow_mut<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key);

        assert!(loan.is_fund_transferred_status<LendCoinType, CollateralCoinType>(), EInvalidLoanStatus);
        assert!(!is_valid_collateral_amount<LendCoinType, CollateralCoinType>(
			configuration.min_health_ratio(),
            loan.amount<LendCoinType, CollateralCoinType>(), 
            loan.collateral_amount<LendCoinType, CollateralCoinType>(), 
            lend_asset, 
            collateral_asset, 
            price_info_object_lending, 
            price_info_object_collateral, 
            clock,
        ), ECanNotLiquidateValidCollateral);

        let (liquidating_price, _, _) = price_info_object_collateral.get_price(collateral_asset.max_price_age_seconds(), clock);

		let collateral_amount = loan.collateral_amount<LendCoinType, CollateralCoinType>();
        let collateral_coin = loan.sub_collateral_balance<LendCoinType, CollateralCoinType>(collateral_amount).to_coin(ctx);
        transfer::public_transfer(collateral_coin, configuration.hot_wallet());

        loan.start_liquidate_loan_offer<LendCoinType, CollateralCoinType>(
			liquidating_price,
			current_timestamp,
        );
    }

    public entry fun start_liquidate_loan_offer_expired<LendCoinType, CollateralCoinType>(
        _: &OperatorCap,
        version: &Version,
        configuration: &Configuration,
        state: &mut State,
        loan_id: ID,
        clock: &Clock,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();
        let current_timestamp = clock.timestamp_ms();

        let loan_key = loan_registry::new_loan_key<LendCoinType, CollateralCoinType>(loan_id);
        assert!(state.contain<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key), ELoanNotFound);
        let loan = state.borrow_mut<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key);      

        assert!(loan.is_fund_transferred_status<LendCoinType, CollateralCoinType>(), EInvalidLoanStatus);
        assert!(loan.is_expired_loan_offer<LendCoinType, CollateralCoinType>(current_timestamp), ECanNotLiquidateUnexpiredLoan);
		
		let collateral_amount = loan.collateral_amount<LendCoinType, CollateralCoinType>();
        let collateral_coin = loan.sub_collateral_balance<LendCoinType, CollateralCoinType>(collateral_amount).to_coin(ctx);
        transfer::public_transfer(collateral_coin, configuration.hot_wallet());

        loan.start_liquidate_loan_offer<LendCoinType, CollateralCoinType>(
			0,
			current_timestamp,
        );
    }

    public entry fun system_liquidate_loan_offer<LendCoinType, CollateralCoinType>(
        _: &OperatorCap,
        version: &Version,
        configuration: &Configuration,
        custodian: &mut Custodian<LendCoinType>,
        state: &mut State,
        loan_id: ID,
        remaining_fund: Coin<LendCoinType>,
        collateral_swapped_amount: u64,
        liquidated_price: u64,
        liquidated_tx: String,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();

        let loan_key = loan_registry::new_loan_key<LendCoinType, CollateralCoinType>(loan_id);
        assert!(state.contain<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key), ELoanNotFound);
        let loan = state.borrow_mut<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key);
		
		assert!(loan.is_liquidating_status<LendCoinType, CollateralCoinType>(), EInvalidLoanStatus);

        let interest_amount = ((loan.amount() * loan.interest() / default_rate_factor() * loan.duration() as u128) / (seconds_in_year() as u128) as u64);
        assert!(collateral_swapped_amount == remaining_fund.value<LendCoinType>() + loan.amount() + interest_amount, EInvalidCoinInput);

        let borrower_fee_amount = ((interest_amount * configuration.borrower_fee_percent() as u128) / (default_rate_factor() as u128) as u64 );
        let refund_to_borrower_amount = remaining_fund.value<LendCoinType>() - borrower_fee_amount;

        let mut remaining_fund_balance = remaining_fund.into_balance<LendCoinType>();
        let borrower_fee_balance = remaining_fund_balance.split<LendCoinType>(borrower_fee_amount);
        transfer::public_transfer(remaining_fund_balance.to_coin<LendCoinType>(ctx), loan.borrower());
        custodian.add_treasury_balance<LendCoinType>(borrower_fee_balance);

        loan.finish_liquidate_loan_offer(
            collateral_swapped_amount,
			refund_to_borrower_amount,
            liquidated_price,
            liquidated_tx,
        );
    }
}