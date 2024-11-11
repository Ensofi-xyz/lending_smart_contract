module enso_lending::operator {
    use std::string::String;
    use wormhole::state::{State as WormholeState};
    use enso_lending::{
        version::{Version},
        configuration::{Self, Configuration},
        state::{Self, State},
        custodian,
        asset_tier::{Self, AssetTierKey, AssetTier},
        asset::{Self, Asset},
        wormhole,
        foreign_chain::{Self, ForeignChain, ForeignChainKey},
        utils,
    };

    const EAssetAlreadyExist: u64 = 1;

    public struct OperatorCap has key, store {
        id: UID
    }

    fun init(ctx: &mut TxContext) {
        let operator_cap = OperatorCap {
            id: object::new(ctx),
        };

        transfer::transfer(operator_cap, @operator);
    }

    entry fun init_system<T>(
        _: &OperatorCap,
        version: &Version,
        lender_fee_percent: u64,
        borrower_fee_percent: u64,
        max_offer_interest: u64,
        min_health_ratio: u64, 
        hot_wallet: address,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();
        configuration::new(
            lender_fee_percent,
            borrower_fee_percent,
            max_offer_interest,
            min_health_ratio,
            hot_wallet,
            ctx,
        );
        state::new(ctx);
        custodian::new<T>(ctx);
    }

    entry fun init_wormhole_emitter(
        _: &OperatorCap,
        version: &Version,
        wormhole_state: &WormholeState,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();

        wormhole::new(wormhole_state, ctx);
    }
 
    entry fun update_configuration(
        _: &OperatorCap,
        version: &Version,
        configuration: &mut Configuration,
        lender_fee_percent: u64,
        borrower_fee_percent: u64,
        max_offer_interest: u64,
        min_health_ratio: u64, 
        hot_wallet: address,
    ) {
        version.assert_current_version();
        configuration.update(
            lender_fee_percent,
            borrower_fee_percent,
            max_offer_interest,
            min_health_ratio,
            hot_wallet,
        );
    }

    entry fun add_asset<CoinType>(
        _: &OperatorCap,
        version: &Version,
        configuration: &mut Configuration,
        name: String,
        symbol: String,
        decimals: u8,
		is_lend_coin: bool,
		is_collateral_coin: bool,
		price_feed_id: String,
		max_price_age_seconds: u64,
		ctx: &mut TxContext
    ) {
        version.assert_current_version();

        let coin_type = utils::get_type<CoinType>();
        assert!(!configuration.contain<String, Asset<CoinType>>(coin_type), EAssetAlreadyExist);
        let asset = asset::new<CoinType>(
            name,
            symbol,
            decimals,
            is_lend_coin,
            is_collateral_coin,
            price_feed_id,
            max_price_age_seconds,
            ctx,
        );
        configuration.add<String, Asset<CoinType>>(coin_type, asset);
    }

    entry fun update_asset<CoinType>(
        _: &OperatorCap,
        version: &Version,
        configuration: &mut Configuration,
        price_feed_id: String,
        max_price_age_seconds: u64,
    ) {
        version.assert_current_version();

        let coin_type = utils::get_type<CoinType>();
        let asset = configuration.borrow_mut<String, Asset<CoinType>>(coin_type);
        asset.update<CoinType>(price_feed_id, max_price_age_seconds);
    }

    entry fun delete_asset<CoinType>(
        _: &OperatorCap,
        version: &Version,
        configuration: &mut Configuration,
    ) {
        version.assert_current_version();

        let coin_type = utils::get_type<CoinType>();
        let asset = configuration.remove<String, Asset<CoinType>>(coin_type);
        asset.delete<CoinType>();
    }

    entry fun add_foreign_chain(
        _: &OperatorCap,
        version: &Version,
        configuration: &mut Configuration,
        chain_id: u16,
        chain_address: vector<u8>,
        emitter_address: vector<u8>,
    ) {
        version.assert_current_version();
        let foreign_chain_key = foreign_chain::new_foreign_chain_key(chain_id);
        let foreign_chain = foreign_chain::new_foreign_chain(chain_address, emitter_address);
        configuration.add<ForeignChainKey, ForeignChain>(foreign_chain_key, foreign_chain);
    }

    entry fun update_foreign_chain(
        _: &OperatorCap,
        version: &Version,
        configuration: &mut Configuration,
        chain_id: u16,
        chain_address: vector<u8>,
        emitter_address: vector<u8>,
    ) {
        version.assert_current_version();
        let foreign_chain_key = foreign_chain::new_foreign_chain_key(chain_id);
        let foreign_chain = configuration.borrow_mut<ForeignChainKey, ForeignChain>(foreign_chain_key);
        foreign_chain.update_foreign_chain(chain_address, emitter_address);
    }

    entry fun delete_foreign_chain(
        _: &OperatorCap,
        version: &Version,
        configuration: &mut Configuration,
        chain_id: u16,
    ) {
        version.assert_current_version();
        let foreign_chain_key = foreign_chain::new_foreign_chain_key(chain_id);

        configuration.remove<ForeignChainKey, ForeignChain>(foreign_chain_key);
    }

    entry fun init_asset_tier<T>(
        _: &OperatorCap,
        version: &Version,
        state: &mut State,
        name: String,
        amount: u64,
        duration: u64,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();

        let lend_token = utils::get_type<T>();
        let asset_tier = asset_tier::new<T>(
            name,
            amount,
            duration,
            lend_token,
            ctx,
        );
        let asset_tier_key = asset_tier::new_asset_tier_key<T>(name);
        state.add<AssetTierKey<T>, AssetTier<T>>(asset_tier_key, asset_tier);
    }

    entry fun update_asset_tier<T>(
        _: &OperatorCap,
        version: &Version,
        state: &mut State,
        name: String,
        amount: u64,
        duration: u64,
    ) {
        version.assert_current_version();

        let asset_tier_key = asset_tier::new_asset_tier_key<T>(name);
        let asset_tier = state.borrow_mut<AssetTierKey<T>, AssetTier<T>>(asset_tier_key);

        asset_tier.update<T>(
            amount,
            duration,
        );
    }

    entry fun delete_asset_tier<T>(
        _: &OperatorCap,
        version: &Version,
        state: &mut State,
        name: String,
    ) {
        version.assert_current_version();

        let asset_tier_key = asset_tier::new_asset_tier_key<T>(name);
        let asset_tier = state.remove<AssetTierKey<T>, AssetTier<T>>(asset_tier_key);

        asset_tier.delete();
    }

    public(package) fun new_operator(
        user_address: address,
        ctx: &mut TxContext,
    ) {
        let operator_cap = OperatorCap {
            id: object::new(ctx),
        };
        transfer::transfer(operator_cap, user_address);
    }
}