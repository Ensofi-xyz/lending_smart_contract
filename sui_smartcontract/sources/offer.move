module enso_lending::offer {
    use sui::coin::Coin;
    use std::string::String;
    use enso_lending::{
        version::Version,
        configuration::Configuration,
        state::State,
        asset_tier::{Self, AssetTier, AssetTierKey},
        asset::Asset,
        offer_registry::{Self, OfferKey, Offer},
        utils,
    };

    const EInvalidInterestValue: u64 = 1;
    const ENotFoundAssetTier: u64 = 2;
    const ENotEnoughBalanceToCreateOffer: u64 = 3;
    const ENotFoundOfferToCancel: u64 = 4;
    const ENotFoundOfferToEdit: u64 = 5;
    const EInvalidLendCoin: u64 = 6;
    const EInvalidOfferStatus: u64 = 7;
    const ESenderIsNotOfferOwner: u64 = 8;

    public entry fun create_offer<LendCoinType>(
        version: &Version,
        state: &mut State,
        configuration: &Configuration,
        asset_tier_name: String,
        lend_coin: Coin<LendCoinType>,
        interest: u64,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();
        let lender = ctx.sender();
        let lend_asset = configuration.borrow<String, Asset<LendCoinType>>(utils::get_type<LendCoinType>());
        assert!(lend_asset.is_lend_coin<LendCoinType>(), EInvalidLendCoin);
        assert!(interest > 0 && interest <= configuration.max_offer_interest(), EInvalidInterestValue);

        let asset_tier_key = asset_tier::new_asset_tier_key<LendCoinType>(asset_tier_name);
        assert!(state.contain<AssetTierKey<LendCoinType>, AssetTier<LendCoinType>>(asset_tier_key), ENotFoundAssetTier);
        let asset_tier = state.borrow<AssetTierKey<LendCoinType>, AssetTier<LendCoinType>>(asset_tier_key);

        assert!(lend_coin.value() == asset_tier.amount(), ENotEnoughBalanceToCreateOffer);

        let hot_wallet = configuration.hot_wallet();
        transfer::public_transfer(lend_coin, hot_wallet);

        let offer = offer_registry::new_offer<LendCoinType>(asset_tier.id(), asset_tier_name, asset_tier.amount(), asset_tier.duration(), interest, lender, ctx);
        let offer_id = object::id(&offer);
        let offer_key = offer_registry::new_offer_key<LendCoinType>(offer_id);

        state.add<OfferKey<LendCoinType>, Offer<LendCoinType>>(offer_key, offer);
        state.add_offer(offer_id, lender, ctx);
    }

    public entry fun request_cancel_offer<LendCoinType>(
        version: &Version,
        state: &mut State,
        offer_id: ID,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();
        let sender = ctx.sender();
        let offer_key = offer_registry::new_offer_key<LendCoinType>(offer_id);
        assert!(state.contain<OfferKey<LendCoinType>, Offer<LendCoinType>>(offer_key), ENotFoundOfferToCancel);
        let offer = state.borrow_mut<OfferKey<LendCoinType>, Offer<LendCoinType>>(offer_key);
        assert!(sender == offer.lender(), ESenderIsNotOfferOwner);
        assert!(offer.is_created_status(), EInvalidOfferStatus);

        offer.cancel_offer(sender);
    }

    public entry fun edit_offer<LendCoinType>(
        version: &Version,
        configuration: &Configuration,
        state: &mut State,
        offer_id: ID,
        interest: u64,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();
        let sender = ctx.sender();
        let offer_key = offer_registry::new_offer_key<LendCoinType>(offer_id);
        assert!(state.contain<OfferKey<LendCoinType>, Offer<LendCoinType>>(offer_key), ENotFoundOfferToEdit);
        let offer = state.borrow_mut<OfferKey<LendCoinType>, Offer<LendCoinType>>(offer_key);
        assert!(sender == offer.lender(), ESenderIsNotOfferOwner);
        assert!(offer.is_created_status(), EInvalidOfferStatus);
        assert!(interest > 0 && interest <= configuration.max_offer_interest(), EInvalidInterestValue);

        offer.edit_offer(interest, sender);
    }
}