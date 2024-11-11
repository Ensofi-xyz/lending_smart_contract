module enso_lending::loan {
    use std::string::String;
    use sui::{
        coin::Coin,
        balance::Balance,
        clock::Clock,
    };
    use pyth::price_info::PriceInfoObject;
    use enso_lending::{
        offer_registry::{Self, Offer, OfferKey},
        state::State,
        configuration::Configuration,
        custodian::Custodian,
        version::Version,
        loan_registry::{Self, Loan, LoanKey, is_valid_collateral_amount},
        asset::Asset,
        utils::{Self, default_rate_factor, seconds_in_year},
    };

    use fun enso_lending::price_feed::is_valid_price_info_object as PriceInfoObject.is_valid;
    use fun sui::coin::from_balance as Balance.to_coin;

    const EOfferNotFound: u64 = 1;
    const EOfferIsNotActive: u64 = 2;
    const ELoanNotFound: u64 = 3;
    const EPriceInfoObjectLendingIsInvalid: u64 = 4;
    const EPriceInfoObjectCollateralIsInvalid: u64 = 5;
    const ELendCoinIsInvalid: u64 = 6;
    const ECollateralCoinIsInvalid: u64 = 7;
    const EInterestIsInvalid: u64 = 8;
    const ECollateralNotValidToMinHealthRatio: u64 = 9;
    const ESenderIsNotLoanBorrower: u64 = 10;
    const EInvalidLoanStatus: u64 = 11;
    const ENotEnoughBalanceToRepay: u64 = 12;
    const ECollateralIsInsufficient: u64 = 15;
    const ECanNotRepayExpiredLoan: u64 = 16;

    entry fun take_loan<LendCoinType, CollateralCoinType>(
        version: &Version,
        configuration: &Configuration,
        state: &mut State,
        offer_id: ID,
        interest: u64,
        collateral: Coin<CollateralCoinType>,
        price_info_object_lending: &PriceInfoObject,
        price_info_object_collateral: &PriceInfoObject,
        clock: &Clock,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();

        let current_timestamp = clock.timestamp_ms();
        let borrower = ctx.sender();
        let lend_asset = configuration.borrow<String, Asset<LendCoinType>>(utils::get_type<LendCoinType>());
        let collateral_asset = configuration.borrow<String, Asset<CollateralCoinType>>(utils::get_type<CollateralCoinType>());

        assert!(lend_asset.is_lend_coin<LendCoinType>(), ELendCoinIsInvalid);
        assert!(collateral_asset.is_collateral_coin<CollateralCoinType>(), ECollateralCoinIsInvalid);
        assert!(price_info_object_lending.is_valid<LendCoinType>(lend_asset), EPriceInfoObjectLendingIsInvalid);
        assert!(price_info_object_collateral.is_valid<CollateralCoinType>(collateral_asset), EPriceInfoObjectCollateralIsInvalid);
        
        let offer_key = offer_registry::new_offer_key<LendCoinType>(offer_id);
        assert!(state.contain<OfferKey<LendCoinType>, Offer<LendCoinType>>(offer_key), EOfferNotFound);
        let offer = { state.borrow_mut<OfferKey<LendCoinType>, Offer<LendCoinType>>(offer_key) };
        assert!(offer.is_created_status<LendCoinType>(), EOfferIsNotActive);
        assert!(interest == offer.interest(),  EInterestIsInvalid);

        assert!(is_valid_collateral_amount<LendCoinType, CollateralCoinType>(
            configuration.min_health_ratio(),
            offer.amount<LendCoinType>(), 
            collateral.value<CollateralCoinType>(), 
            lend_asset, 
            collateral_asset, 
            price_info_object_lending, 
            price_info_object_collateral, 
            clock,
        ), ECollateralNotValidToMinHealthRatio);

        let loan = loan_registry::new_loan(
            collateral,
            offer,
            borrower,
            current_timestamp,
            ctx,
        );

        let loan_id = object::id(&loan);
        let loan_key = loan_registry::new_loan_key<LendCoinType, CollateralCoinType>(loan_id);

        offer.take_loan();
        state.add<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType,CollateralCoinType>>(loan_key, loan);
        state.add_loan(loan_id, borrower, ctx);
    }

    entry fun repay<LendCoinType, CollateralCoinType>(
        version: &Version,
        configuration: &Configuration,
        custodian: &mut Custodian<LendCoinType>,
        state: &mut State,
        loan_id: ID,
        repay_coin: Coin<LendCoinType>,
        clock: &Clock,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();

        let current_timestamp = clock.timestamp_ms();
        let lend_token = utils::get_type<LendCoinType>();
        let collateral_token = utils::get_type<CollateralCoinType>();

        let loan_key = loan_registry::new_loan_key<LendCoinType, CollateralCoinType>(loan_id);
        assert!(state.contain<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key), ELoanNotFound);
        let loan = state.borrow_mut<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key);
        
        assert!(ctx.sender() == loan.borrower(), ESenderIsNotLoanBorrower);
        assert!(loan.is_fund_transferred_status<LendCoinType, CollateralCoinType>(), EInvalidLoanStatus);
        assert!(loan.start_timestamp() + (loan.duration() * 1000) > current_timestamp, ECanNotRepayExpiredLoan);

        let interest_amount = ((loan.amount() * loan.interest() / default_rate_factor() * loan.duration() as u128) / (seconds_in_year() as u128) as u64);
        let borrower_fee_amount = ((interest_amount * configuration.borrower_fee_percent() as u128) / (default_rate_factor() as u128) as u64 );
        let repay_amount = loan.amount() + borrower_fee_amount + interest_amount;
        assert!(repay_coin.value<LendCoinType>() == repay_amount, ENotEnoughBalanceToRepay);

        let mut repay_balance = repay_coin.into_balance<LendCoinType>();
        let borrower_fee_balance = repay_balance.split<LendCoinType>(borrower_fee_amount);

        transfer::public_transfer(repay_balance.to_coin(ctx), configuration.hot_wallet());
        custodian.add_treasury_balance<LendCoinType>(borrower_fee_balance);

        let collateral_amount = loan.collateral_amount<LendCoinType, CollateralCoinType>();
        let collateral_balance = loan.sub_collateral_balance<LendCoinType, CollateralCoinType>(collateral_amount);
        transfer::public_transfer(collateral_balance.to_coin(ctx), ctx.sender());

        loan.repay(
            repay_amount,
            collateral_amount,
            lend_token,
            collateral_token,
        );
    }

    entry fun withdraw_collateral_loan_offer<LendCoinType, CollateralCoinType>(
        version: &Version,
        configuration: &Configuration,
        state: &mut State,
        loan_id: ID,
        withdraw_amount: u64,
        price_info_object_lending: &PriceInfoObject,
        price_info_object_collateral: &PriceInfoObject,
        clock: &Clock,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();

        let current_timestamp = clock.timestamp_ms();
        let lend_asset = configuration.borrow<String, Asset<LendCoinType>>(utils::get_type<LendCoinType>());
        let collateral_asset = configuration.borrow<String, Asset<CollateralCoinType>>(utils::get_type<CollateralCoinType>());
        
        assert!(price_info_object_lending.is_valid<LendCoinType>(lend_asset), EPriceInfoObjectLendingIsInvalid);
        assert!(price_info_object_collateral.is_valid<CollateralCoinType>(collateral_asset), EPriceInfoObjectCollateralIsInvalid);

        let loan_key = loan_registry::new_loan_key<LendCoinType, CollateralCoinType>(loan_id);
        let loan = state.borrow_mut<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key);

        assert!(loan.is_fund_transferred_status<LendCoinType, CollateralCoinType>(), EInvalidLoanStatus);
        let collateral_amount = loan.collateral_amount<LendCoinType, CollateralCoinType>();
        assert!(collateral_amount >= withdraw_amount, ECollateralIsInsufficient);
        let remaining_collateral_amount = collateral_amount - withdraw_amount;

        assert!(is_valid_collateral_amount<LendCoinType, CollateralCoinType>(
            configuration.min_health_ratio(),
            loan.amount(), 
            remaining_collateral_amount, 
            lend_asset, 
            collateral_asset, 
            price_info_object_lending, 
            price_info_object_collateral, 
            clock,
        ), ECollateralNotValidToMinHealthRatio);

        let collateral_balance = loan.sub_collateral_balance<LendCoinType, CollateralCoinType>(withdraw_amount);
        transfer::public_transfer(collateral_balance.to_coin<CollateralCoinType>(ctx), ctx.sender());

        loan.withdraw_collateral<LendCoinType, CollateralCoinType>(
            withdraw_amount,
            remaining_collateral_amount,
            current_timestamp,
        );
    }

    entry fun deposit_collateral_loan_offer<LendCoinType, CollateralCoinType>(
        version: &Version,
        configuration: &Configuration,
        state: &mut State,
        loan_id: ID,
        deposit_coin: Coin<CollateralCoinType>,
        clock: &Clock,
    ) {
        version.assert_current_version();

        let current_timestamp = clock.timestamp_ms();
        let lend_token = utils::get_type<LendCoinType>();
        let collateral_token = utils::get_type<CollateralCoinType>();
        let loan_key = loan_registry::new_loan_key<LendCoinType, CollateralCoinType>(loan_id);

        let ( offer_id ) = {
            assert!(state.contain<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key), ELoanNotFound);
            let loan = state.borrow<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key);
            loan.offer_id()
        };

        let ( asset_tier ) = {
            let offer_key = offer_registry::new_offer_key<LendCoinType>(offer_id);
            assert!(state.contain<OfferKey<LendCoinType>, Offer<LendCoinType>>(offer_key), EOfferNotFound);
            let offer = state.borrow<OfferKey<LendCoinType>, Offer<LendCoinType>>(offer_key);
            offer.asset_tier()
        };

        let loan = state.borrow_mut<LoanKey<LendCoinType, CollateralCoinType>, Loan<LendCoinType, CollateralCoinType>>(loan_key);

        assert!(loan.is_fund_transferred_status<LendCoinType, CollateralCoinType>(), EInvalidLoanStatus);
        loan.add_collateral_balance<LendCoinType, CollateralCoinType>(deposit_coin.into_balance<CollateralCoinType>());
        let total_collateral_amount = loan.collateral_amount<LendCoinType, CollateralCoinType>();

        loan.deposit_collateral(
            asset_tier,
            configuration.lender_fee_percent(),
            configuration.borrower_fee_percent(),
            lend_token,
            collateral_token,
            total_collateral_amount,
            current_timestamp,
        );
    }
}