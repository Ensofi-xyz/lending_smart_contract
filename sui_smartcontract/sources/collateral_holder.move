module enso_lending::collateral_holder {
	use std::string::String;
	use sui::{
		balance::Balance,
		event,
	};

	use fun std::string::utf8 as vector.to_string;

	const HOLDING_COLLATERAL_DURATION: u64 = 300; //seconds
	const CREATED_STATUS: vector<u8> = b"Created";
	const BORROWER_PAID_STATUS: vector<u8> = b"BorrowerPaid";
	const LIQUIDATED_STATUS: vector<u8> = b"Liquidated";

	public struct CollateralHolderKey has store, copy, drop {
		offer_id: String,
		lend_chain_borrower: String,
	}

	public struct CollateralHolder<phantom CollateralCoinType> has key, store {
		id: UID,
		offer_id: String,
		lend_amount: u64,
		borrower: address,
		collateral: Balance<CollateralCoinType>,
		start_timestamp: u64,
		lend_chain: u16,
		status: String,
	}

	public struct CollateralCreatedEvent has copy, drop {
		offer_id: String,
		lend_amount: u64,
		borrower: address,
		collateral_amount: u64,
		start_timestamp: u64,
		lend_chain: u16,
		status: String,
	}

	public struct CollateralCancelledEvent has copy, drop {
		offer_id: String,
		borrower: address,
		lend_chain: u16,
	}

	public struct RefundCollateralEvent has copy, drop {
		offer_id: String,
		borrower: address,
		lend_chain: u16,
	}

	public struct DepositCollateralEvent has copy, drop {
		offer_id: String,
		borrower: address,
		collateral_amount: u64,
		collateral_token: String,
		lend_chain: u16,
	}

	public struct WithdrawCollateralEvent has copy, drop {
		offer_id: String,
		borrower: address,
		withdraw_amount: u64,
		remaining_collateral_amount: u64,
	}

	public struct LiquidateCollateralEvent has copy, drop {
		offer_id: String,
		borrower: address,
		lend_chain: u16,
		liquidating_price: u64,
		liquidating_at: u64,
	}

	public(package) fun new<CollateralCoinType>(
		offer_id: String,
		lend_amount: u64,
		borrower: address,
		collateral: Balance<CollateralCoinType>,
		start_timestamp: u64,
		lend_chain: u16,
		ctx: &mut TxContext,
	): CollateralHolder<CollateralCoinType> {
		let collateral_amount = collateral.value<CollateralCoinType>();
		let collateral_holder = CollateralHolder<CollateralCoinType> {
			id: object::new(ctx),
			offer_id,
			lend_amount,
			borrower,
			collateral,
			start_timestamp,
			lend_chain,
			status: CREATED_STATUS.to_string()
		};

		event::emit(CollateralCreatedEvent {
			offer_id,
			lend_amount,
			borrower,
			collateral_amount,
			start_timestamp,
			lend_chain,
			status: collateral_holder.status,
		});

		collateral_holder
	}

	public(package) fun delete<CollateralCoinType>(
		collateral_holder: CollateralHolder<CollateralCoinType>,
	): Balance<CollateralCoinType> {
		let CollateralHolder {
			id, offer_id, lend_amount: _, borrower, collateral, start_timestamp:_, lend_chain, status: _,
		} = collateral_holder;

		object::delete(id);

		event::emit(CollateralCancelledEvent {
			offer_id,
			borrower,
			lend_chain
		});

		collateral
	}

	public(package) fun deposit_collateral<CollateralCoinType>(
		collateral_holder: &CollateralHolder<CollateralCoinType>,
		collateral_token: String,
	) {
		event::emit(DepositCollateralEvent {
			offer_id: collateral_holder.offer_id,
			borrower: collateral_holder.borrower,
			collateral_amount: collateral_holder.collateral.value(),
			collateral_token,
			lend_chain: collateral_holder.lend_chain,
		});
	}

	public(package) fun withdraw_collateral<CollateralCoinType>(
		collateral_holder: &CollateralHolder<CollateralCoinType>,
		withdraw_amount: u64,
		remaining_collateral_amount: u64,
	) {
		event::emit(WithdrawCollateralEvent {
			offer_id: collateral_holder.offer_id,
			borrower: collateral_holder.borrower,
			withdraw_amount,
			remaining_collateral_amount,
		})
	}

	public(package) fun liquidate_collateral<CollateralCoinType>(
		collateral_holder: &mut CollateralHolder<CollateralCoinType>,
		liquidating_price: u64,
		liquidating_at: u64,
	) {
		collateral_holder.status = LIQUIDATED_STATUS.to_string();
		event::emit(LiquidateCollateralEvent {
			offer_id: collateral_holder.offer_id,
			borrower: collateral_holder.borrower,
			lend_chain: collateral_holder.lend_chain,
			liquidating_price,
			liquidating_at,
		})
	}

	public(package) fun refund_collateral_to_repaid_borrower<CollateralCoinType>(
		collateral_holder: &mut CollateralHolder<CollateralCoinType>
	) {
		collateral_holder.status = BORROWER_PAID_STATUS.to_string();
		event::emit(RefundCollateralEvent {
			offer_id: collateral_holder.offer_id,
			borrower: collateral_holder.borrower,
			lend_chain: collateral_holder.lend_chain,
		});
	}

	public(package) fun add_collateral_balance<CollateralCoinType>(
		collateral_holder: &mut CollateralHolder<CollateralCoinType>,
		amount: Balance<CollateralCoinType>,
	) {
		collateral_holder.collateral.join<CollateralCoinType>(amount);
	}

	public(package) fun sub_collateral_balance<CollateralCoinType>(
		collateral_holder: &mut CollateralHolder<CollateralCoinType>,
		amount: u64
	): Balance<CollateralCoinType> {
		collateral_holder.collateral.split<CollateralCoinType>(amount)
	}

	public fun new_holder_key(offer_id: String, lend_chain_borrower: String): CollateralHolderKey {
		CollateralHolderKey {
			offer_id,
			lend_chain_borrower,
		}
	}

	public fun is_created_status<CollateralCoinType>(
		collateral_holder: &CollateralHolder<CollateralCoinType>
	): bool {
		collateral_holder.status == CREATED_STATUS.to_string()
	}

	public fun lend_amount<CollateralCoinType>(
		collateral_holder: &CollateralHolder<CollateralCoinType>
	): u64 {
		collateral_holder.lend_amount
	}

	public fun collateral_amount<CollateralCoinType>(
		collateral_holder: &CollateralHolder<CollateralCoinType>
	): u64 {
		collateral_holder.collateral.value<CollateralCoinType>()
	}

	public fun borrower<CollateralCoinType>(
		collateral_holder: &CollateralHolder<CollateralCoinType>
	): address {
		collateral_holder.borrower
	}

	public fun start_timestamp<CollateralCoinType>(
		collateral_holder: &CollateralHolder<CollateralCoinType>
	): u64 {
		collateral_holder.start_timestamp
	}

	public fun lend_chain<CollateralCoinType>(
		collateral_holder: &CollateralHolder<CollateralCoinType>,
	): u16 {
		collateral_holder.lend_chain
	}

	public fun holding_collateral_duration(): u64 {
		HOLDING_COLLATERAL_DURATION
	}
}