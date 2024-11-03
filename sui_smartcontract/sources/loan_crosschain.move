module enso_lending::loan_crosschain {
	use std::string::String;
	use sui::{
		sui::SUI,
		balance::Balance,
        coin::Coin,
        clock::Clock,
    };
	use pyth::price_info::PriceInfoObject;
	use wormhole::state::{State as WormholeState};
    use enso_lending::{
        state::State,
        configuration::Configuration,
        version::Version,
		loan_registry::is_valid_collateral_amount,
        asset::Asset,
		collateral_holder::{Self, CollateralHolderKey, CollateralHolder},
		wormhole::{Self, ProtectedEC, parse_and_verify_vaa},
        utils::{get_type, get_emitter_address_by_chain},
		vaa_utils,
    };

	use fun enso_lending::price_feed::is_valid_price_info_object as PriceInfoObject.is_valid;
	use fun enso_lending::utils::u64_to_string as u64.to_string;
	use fun std::string::utf8 as vector.to_string;
	use fun sui::coin::from_balance as Balance.to_coin;

	const ELendCoinIsInvalid: u64 = 1;
    const ECollateralCoinIsInvalid: u64 = 2;
	const EPriceInfoObjectLendingIsInvalid: u64 = 3;
    const EPriceInfoObjectCollateralIsInvalid: u64 = 4;
	const EAlreadyDepositedCollateralForThisOffer: u64 = 5;
	const ECollateralNotValidToMinHealthRatio: u64 = 6;
	const ECollateralHolderNotFound: u64 = 7;
	const EMismatchDataWithVaa: u64 = 8;
	const ESenderIsInvalid: u64 = 9;
	const EInvalidCollateralHolderStatus: u64 = 10;
	const ECollateralIsInsufficient: u64 = 11;

	public entry fun deposit_collateral_to_take_loan<LendCoinType, CollateralCoinType>(
		version: &Version,
		state: &mut State,
		configuration: &Configuration,
		protected_ec: &mut ProtectedEC,
		wormhole_state: &mut WormholeState,
		message_fee: Coin<SUI>,
		collateral: Coin<CollateralCoinType>,
		price_info_object_lending: &PriceInfoObject,
		price_info_object_collateral: &PriceInfoObject,
		tier_id: vector<u8>,
		offer_id: vector<u8>,
		lend_chain_borrower: vector<u8>,
		lend_amount: u64,
		target_chain: u64,
		target_address: vector<u8>,
		clock: &Clock,
		ctx: &mut TxContext,
	) {
		version.assert_current_version();
		
		let current_timestamp = clock.timestamp_ms();
		let borrower = ctx.sender();
		let collateral_amount = collateral.value<CollateralCoinType>();
		let lend_asset = configuration.borrow<String, Asset<LendCoinType>>(get_type<LendCoinType>());
		let collateral_asset = configuration.borrow<String, Asset<CollateralCoinType>>(get_type<CollateralCoinType>());

        assert!(lend_asset.is_lend_coin<LendCoinType>(), ELendCoinIsInvalid);
        assert!(collateral_asset.is_collateral_coin<CollateralCoinType>(), ECollateralCoinIsInvalid);
        assert!(price_info_object_lending.is_valid<LendCoinType>(lend_asset), EPriceInfoObjectLendingIsInvalid);
        assert!(price_info_object_collateral.is_valid<CollateralCoinType>(collateral_asset), EPriceInfoObjectCollateralIsInvalid);

        assert!(is_valid_collateral_amount<LendCoinType, CollateralCoinType>(
            configuration.min_health_ratio(),
            lend_amount, 
            collateral.value<CollateralCoinType>(), 
            lend_asset, 
            collateral_asset, 
            price_info_object_lending, 
            price_info_object_collateral, 
            clock,
        ), ECollateralNotValidToMinHealthRatio);

		let collateral_holder_key = collateral_holder::new_holder_key(offer_id.to_string(), lend_chain_borrower.to_string());
		assert!(!state.contain<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key), EAlreadyDepositedCollateralForThisOffer);
		let collateral_holder = collateral_holder::new<CollateralCoinType>(
			offer_id.to_string(),
			lend_amount,
			borrower,
			collateral.into_balance<CollateralCoinType>(),
			current_timestamp,
			(target_chain as u16),
			ctx,
		);

		let payload = gen_collateral_created_message_payload(
			target_chain,
			target_address,
			tier_id,
			offer_id,
			collateral_amount,
			*get_type<CollateralCoinType>().bytes(),
			collateral_asset.decimals() as u64,
			*collateral_asset.symbol().bytes(),
			lend_chain_borrower,
		);

		wormhole::send_message(
            protected_ec,
            wormhole_state,
            payload,
            message_fee,
            clock,
        );

		state.add<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key, collateral_holder);
	}

	public entry fun cancel_collateral<CollateralCoinType>(
		version: &Version,
		state: &mut State,
		wormhole_state: &WormholeState,
		offer_id: vector<u8>,
		lend_chain_borrower: vector<u8>,
		vaa_buf: vector<u8>,
		clock: &Clock,
		ctx: &mut TxContext,
	) {
		version.assert_current_version();
		let sender = ctx.sender();

		let collateral_holder_key = collateral_holder::new_holder_key(offer_id.to_string(), lend_chain_borrower.to_string());
		assert!(state.contain<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key), ECollateralHolderNotFound);
		let collateral_holder = state.remove<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key);
		let borrower = collateral_holder.borrower();

		assert!(collateral_holder.is_created_status(), EInvalidCollateralHolderStatus);
		assert!(sender == borrower, ESenderIsInvalid);

		let payload_body = parse_and_verify_vaa(
			wormhole_state,
			vaa_buf,
			collateral_holder.lend_chain(),
			get_emitter_address_by_chain(collateral_holder.lend_chain()),
			b"cancel_collateral",
			clock,
		);

		let (parsed_offer_id, parsed_lend_chain_borrower) = vaa_utils::parse_cancel_collateral_payload(payload_body);
		assert!(offer_id.to_string() == parsed_offer_id && lend_chain_borrower.to_string() == parsed_lend_chain_borrower, EMismatchDataWithVaa);

		let refund_collateral_balance = collateral_holder.delete();
		transfer::public_transfer(refund_collateral_balance.to_coin(ctx), borrower);
	}

	public entry fun claim_refund_collateral<CollateralCoinType>(
		version: &Version,
		state: &mut State,
		wormhole_state: &WormholeState,
		offer_id: vector<u8>,
		lend_chain_borrower: vector<u8>,
		vaa_buf: vector<u8>,
		clock: &Clock,
		ctx: &mut TxContext,
	) {
		version.assert_current_version();
		let sender = ctx.sender();

		let collateral_holder_key = collateral_holder::new_holder_key(offer_id.to_string(), lend_chain_borrower.to_string());
		assert!(state.contain<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key), ECollateralHolderNotFound);
		let collateral_holder = state.borrow_mut<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key);
		let borrower = collateral_holder.borrower();

		assert!(collateral_holder.is_created_status(), EInvalidCollateralHolderStatus);
		assert!(sender == borrower, ESenderIsInvalid);

		let payload_body = parse_and_verify_vaa(
			wormhole_state,
			vaa_buf,
			collateral_holder.lend_chain(),
			get_emitter_address_by_chain(collateral_holder.lend_chain()),
			b"refund_collateral_to_repaid_borrower",
			clock,
		);

		let (parsed_offer_id, parsed_lend_chain_borrower) = vaa_utils::parse_refund_collateral_to_repaid_borrower_payload(payload_body);

		assert!(offer_id.to_string() == parsed_offer_id && lend_chain_borrower.to_string() == parsed_lend_chain_borrower, EMismatchDataWithVaa);
		let collateral_amount = collateral_holder.collateral_amount<CollateralCoinType>();
		let collateral_balance = collateral_holder.sub_collateral_balance<CollateralCoinType>(collateral_amount);
		transfer::public_transfer(collateral_balance.to_coin(ctx), collateral_holder.borrower());

		collateral_holder.refund_collateral_to_repaid_borrower();
	}

	public entry fun deposit_collateral<CollateralCoinType>(
		version: &Version,
		state: &mut State,
		configuration: &Configuration,
		protected_ec: &mut ProtectedEC,
		wormhole_state: &mut WormholeState,
		message_fee: Coin<SUI>,
		offer_id: vector<u8>,
		lend_chain_borrower: vector<u8>,
		deposit_coin: Coin<CollateralCoinType>,
		clock: &Clock,
		ctx: &mut TxContext,
	) {
		version.assert_current_version();
		let sender = ctx.sender();
		
		let collateral_asset = configuration.borrow<String, Asset<CollateralCoinType>>(get_type<CollateralCoinType>());
		let collateral_holder_key = collateral_holder::new_holder_key(offer_id.to_string(), lend_chain_borrower.to_string());
		assert!(state.contain<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key), ECollateralHolderNotFound);
		let collateral_holder = state.borrow_mut<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key);
		let borrower = collateral_holder.borrower();

		assert!(collateral_holder.is_created_status(), EInvalidCollateralHolderStatus);
		assert!(sender == borrower, ESenderIsInvalid);

	 	collateral_holder.add_collateral_balance<CollateralCoinType>(deposit_coin.into_balance());
		collateral_holder.deposit_collateral<CollateralCoinType>(get_type<CollateralCoinType>());

		let payload = gen_deposit_collateral_message_payload(
			(collateral_holder.lend_chain() as u64),
			get_emitter_address_by_chain(collateral_holder.lend_chain()),
			offer_id,
			collateral_holder.collateral_amount(),
			*get_type<CollateralCoinType>().bytes(),
			(collateral_asset.decimals() as u64)
		);
		
		wormhole::send_message(
            protected_ec,
            wormhole_state,
            payload,
            message_fee,
            clock,
        );
	}

	public entry fun withdraw_collateral<LendCoinType, CollateralCoinType>(
		version: &Version,
		state: &mut State,
		configuration: &Configuration,
		protected_ec: &mut ProtectedEC,
		wormhole_state: &mut WormholeState,
		message_fee: Coin<SUI>,
		offer_id: vector<u8>,
		lend_chain_borrower: vector<u8>,
		withdraw_amount: u64,
		price_info_object_lending: &PriceInfoObject,
		price_info_object_collateral: &PriceInfoObject,
		clock: &Clock,
		ctx: &mut TxContext,
	) {
		version.assert_current_version();
		let sender = ctx.sender();

		let lend_asset = configuration.borrow<String, Asset<LendCoinType>>(get_type<LendCoinType>());
		let collateral_asset = configuration.borrow<String, Asset<CollateralCoinType>>(get_type<CollateralCoinType>());
		assert!(price_info_object_lending.is_valid<LendCoinType>(lend_asset), EPriceInfoObjectLendingIsInvalid);
        assert!(price_info_object_collateral.is_valid<CollateralCoinType>(collateral_asset), EPriceInfoObjectCollateralIsInvalid);

		let collateral_holder_key = collateral_holder::new_holder_key(offer_id.to_string(), lend_chain_borrower.to_string());
		assert!(state.contain<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key), ECollateralHolderNotFound);
		let collateral_holder = state.borrow_mut<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key);
		let borrower = collateral_holder.borrower();
		let collateral_amount = collateral_holder.collateral_amount();

		assert!(collateral_holder.is_created_status(), EInvalidCollateralHolderStatus);
		assert!(sender == borrower, ESenderIsInvalid);
		assert!(collateral_amount >= withdraw_amount, ECollateralIsInsufficient);

		let remaining_collateral_amount = collateral_amount - withdraw_amount;
		assert!(is_valid_collateral_amount<LendCoinType, CollateralCoinType>(
            configuration.min_health_ratio(),
            collateral_holder.lend_amount(), 
            remaining_collateral_amount, 
            lend_asset, 
            collateral_asset, 
            price_info_object_lending, 
            price_info_object_collateral, 
            clock,
        ), ECollateralNotValidToMinHealthRatio);

		let collateral_balance = collateral_holder.sub_collateral_balance<CollateralCoinType>(withdraw_amount);
        transfer::public_transfer(collateral_balance.to_coin<CollateralCoinType>(ctx), ctx.sender());

        collateral_holder.withdraw_collateral<CollateralCoinType>(
            withdraw_amount,
            remaining_collateral_amount,
        );

		let payload = gen_withdraw_collateral_message_payload(
			(collateral_holder.lend_chain() as u64),
			get_emitter_address_by_chain(collateral_holder.lend_chain()),
			offer_id,
			withdraw_amount,
			remaining_collateral_amount,
			*get_type<CollateralCoinType>().bytes(),
			(collateral_asset.decimals() as u64),
		);

		wormhole::send_message(
            protected_ec,
            wormhole_state,
            payload,
            message_fee,
            clock,
        );
	}

	fun gen_collateral_created_message_payload(
        target_chain: u64,
        target_address: vector<u8>,
        tier_id: vector<u8>,
        offer_id: vector<u8>,
        collateral_amount: u64,
        collateral_coin_type: vector<u8>,
        collateral_decimal: u64,
		collateral_coin_symbol: vector<u8>,
		lend_chain_borrower: vector<u8>,
    ): vector<u8> {
        let mut payload: vector<u8> = vector[];
        vector::append(&mut payload, target_chain.to_string());
        vector::append(&mut payload, b",");
        vector::append(&mut payload, target_address);
        vector::append(&mut payload, b",");
        vector::append(&mut payload, b"create_loan_offer_crosschain");
        vector::append(&mut payload, b",");
        vector::append(&mut payload, tier_id);
        vector::append(&mut payload, b",");
        vector::append(&mut payload, offer_id);
        vector::append(&mut payload, b",");
        vector::append(&mut payload, collateral_amount.to_string());
        vector::append(&mut payload, b",");
        vector::append(&mut payload, collateral_coin_type);
        vector::append(&mut payload, b",");
        vector::append(&mut payload, collateral_decimal.to_string());
		vector::append(&mut payload, b",");
		vector::append(&mut payload, collateral_coin_symbol);
		vector::append(&mut payload, b",");
		vector::append(&mut payload, lend_chain_borrower);

        payload
    }

	fun gen_deposit_collateral_message_payload(
        target_chain: u64,
        target_address: vector<u8>,
        offer_id: vector<u8>,
        collateral_amount: u64,
        pyth_collateral_symbol: vector<u8>,
        collateral_decimal: u64,
    ): vector<u8> {
        let mut payload: vector<u8> = vector[];
        vector::append(&mut payload, target_chain.to_string());
        vector::append(&mut payload, b",");
        vector::append(&mut payload, target_address);
        vector::append(&mut payload, b",");
        vector::append(&mut payload, b"sync_deposit_collateral_crosschain");
        vector::append(&mut payload, b",");
        vector::append(&mut payload, offer_id);
        vector::append(&mut payload, b",");
        vector::append(&mut payload, collateral_amount.to_string());
        vector::append(&mut payload, b",");
        vector::append(&mut payload, pyth_collateral_symbol);
        vector::append(&mut payload, b",");
        vector::append(&mut payload, collateral_decimal.to_string());

        payload
    }

	fun gen_withdraw_collateral_message_payload(
        target_chain: u64,
        target_address: vector<u8>,
        offer_id: vector<u8>,
        withdraw_amount: u64,
		remaining_collateral_amount: u64,
        pyth_collateral_symbol: vector<u8>,
        collateral_decimal: u64,
    ): vector<u8> {
        let mut payload: vector<u8> = vector[];
        vector::append(&mut payload, target_chain.to_string());
        vector::append(&mut payload, b",");
        vector::append(&mut payload, target_address);
        vector::append(&mut payload, b",");
        vector::append(&mut payload, b"sync_withdraw_collateral_crosschain");
        vector::append(&mut payload, b",");
        vector::append(&mut payload, offer_id);
        vector::append(&mut payload, b",");
        vector::append(&mut payload, withdraw_amount.to_string());
		vector::append(&mut payload, b",");
		vector::append(&mut payload, remaining_collateral_amount.to_string());
        vector::append(&mut payload, b",");
        vector::append(&mut payload, pyth_collateral_symbol);
        vector::append(&mut payload, b",");
        vector::append(&mut payload, collateral_decimal.to_string());

        payload
    }

}