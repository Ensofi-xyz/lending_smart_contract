module enso_lending::asset {
	use std::string::String;

	public struct Asset<phantom LendCoinType> has key, store {
		id: UID,
		name: String,
		symbol: String,
		decimals: u8,
		is_lend_coin: bool,
		is_collateral_coin: bool,
		price_feed_id: String,
		max_price_age_seconds: u64,
	}

	public(package) fun new<LendCoinType>(
		name: String,
		symbol: String,
		decimals: u8,
		is_lend_coin: bool,
		is_collateral_coin: bool,
		price_feed_id: String,
		max_price_age_seconds: u64,
		ctx: &mut TxContext
	): Asset<LendCoinType> {
		Asset<LendCoinType> {
			id: object::new(ctx),
			name,
			symbol,
			decimals,
			is_lend_coin,
			is_collateral_coin,
			price_feed_id,
			max_price_age_seconds,
		}
	}

	public(package) fun update<LendCoinType>(
		asset: &mut Asset<LendCoinType>,
		price_feed_id: String,
		max_price_age_seconds: u64,
	) {
		asset.price_feed_id = price_feed_id;
		asset.max_price_age_seconds = max_price_age_seconds;
	}

	public(package) fun delete<LendCoinType>(
		asset: Asset<LendCoinType>
	) {
		let Asset<LendCoinType> {
			id, name: _,symbol: _, decimals:_, is_lend_coin:_, is_collateral_coin:_, price_feed_id:_, max_price_age_seconds:_
		} = asset;
		object::delete(id);
	}

	public fun symbol<LendCoinType>(
		asset: &Asset<LendCoinType>
	): String {
		asset.symbol
	}

	public fun decimals<LendCoinType>(
		asset: &Asset<LendCoinType>
	): u8 {
		asset.decimals
	}

	public fun is_lend_coin<LendCoinType>(
		asset: &Asset<LendCoinType>
	): bool {
		asset.is_lend_coin
	}

	public fun is_collateral_coin<LendCoinType>(
		asset: &Asset<LendCoinType>
	): bool {
		asset.is_collateral_coin
	}

	public fun price_feed_id<LendCoinType>(
		asset: &Asset<LendCoinType>
	): String {
		asset.price_feed_id
	}

	public fun max_price_age_seconds<LendCoinType>(
		asset: &Asset<LendCoinType>
	): u64 {
		asset.max_price_age_seconds
	}
}