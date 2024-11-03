module enso_lending::operator_loan_crosschain {
	use sui::{
		balance::Balance,
        clock::Clock,
    };
	use wormhole::state::{State as WormholeState};
    use enso_lending::{
		operator::OperatorCap,
        state::State,
        configuration::Configuration,
        version::Version,
		collateral_holder::{Self, CollateralHolderKey, CollateralHolder},
		wormhole::parse_and_verify_vaa,
        utils::get_emitter_address_by_chain,
		vaa_utils,
    };
	use fun std::string::utf8 as vector.to_string;
	use fun sui::coin::from_balance as Balance.to_coin;

	const ECollateralHolderNotFound: u64 = 1;
	const EInvalidCollateralHolderStatus: u64 = 2; 
	const EMismatchDataWithVaa: u64 = 3;

	public entry fun system_cancel_collateral<CollateralCoinType>(
		_: &OperatorCap,
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

		let collateral_holder_key = collateral_holder::new_holder_key(offer_id.to_string(), lend_chain_borrower.to_string());
		assert!(state.contain<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key), ECollateralHolderNotFound);
		let collateral_holder = state.remove<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key);
		let borrower = collateral_holder.borrower();

		assert!(collateral_holder.is_created_status(), EInvalidCollateralHolderStatus);

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

	public entry fun refund_collateral_to_repaid_borrower<CollateralCoinType>(
		_: &OperatorCap,
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

		let collateral_holder_key = collateral_holder::new_holder_key(offer_id.to_string(), lend_chain_borrower.to_string());
		assert!(state.contain<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key), ECollateralHolderNotFound);
		let collateral_holder = state.borrow_mut<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key);
		assert!(collateral_holder.is_created_status(), EInvalidCollateralHolderStatus);

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

	public entry fun start_liquidate_collateral<CollateralCoinType>(
		_: &OperatorCap,
		version: &Version,
		configuration: &Configuration,
		state: &mut State,
		wormhole_state: &WormholeState,
		offer_id: vector<u8>,
		lend_chain_borrower: vector<u8>,
		vaa_buf: vector<u8>,
		clock: &Clock,
		ctx: &mut TxContext,
	) {
		version.assert_current_version();

		let collateral_holder_key = collateral_holder::new_holder_key(offer_id.to_string(), lend_chain_borrower.to_string());
		assert!(state.contain<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key), ECollateralHolderNotFound);
		let collateral_holder = state.borrow_mut<CollateralHolderKey, CollateralHolder<CollateralCoinType>>(collateral_holder_key);
		assert!(collateral_holder.is_created_status(), EInvalidCollateralHolderStatus);

		let payload_body = parse_and_verify_vaa(
			wormhole_state,
			vaa_buf,
			collateral_holder.lend_chain(),
			get_emitter_address_by_chain(collateral_holder.lend_chain()),
			b"start_liquidate_collateral",
			clock,
		);

		let (parsed_offer_id, parsed_lend_chain_borrower, liquidating_price, liquidating_at) =
			vaa_utils::parse_liquidate_collateral_payload(payload_body);
		
		assert!(offer_id.to_string() == parsed_offer_id && lend_chain_borrower.to_string() == parsed_lend_chain_borrower, EMismatchDataWithVaa);

		let collateral_amount = collateral_holder.collateral_amount<CollateralCoinType>();
		let collateral_balance = collateral_holder.sub_collateral_balance<CollateralCoinType>(collateral_amount);
		transfer::public_transfer(collateral_balance.to_coin(ctx), configuration.hot_wallet());

		collateral_holder.liquidate_collateral(liquidating_price, liquidating_at);
	}
}