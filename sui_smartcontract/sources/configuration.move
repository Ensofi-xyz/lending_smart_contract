module enso_lending::configuration {
    use sui::{
        dynamic_field as field,
    };

    public struct Configuration has key, store {
        id: UID,
        lender_fee_percent: u64,
        borrower_fee_percent: u64,
        max_offer_interest: u64,
        min_health_ratio: u64,
        hot_wallet: address,
    }

    public(package) fun new(
        lender_fee_percent: u64,
        borrower_fee_percent: u64,
        max_offer_interest: u64,
        min_health_ratio: u64,
        hot_wallet: address,
        ctx: &mut TxContext,
    ) {
        let configuration = Configuration {
            id: object::new(ctx),
            lender_fee_percent,
            borrower_fee_percent,
            max_offer_interest,
            min_health_ratio,
            hot_wallet,
        };
        transfer::public_share_object(configuration);
    }

    public(package) fun update(
        configuration: &mut Configuration,
        lender_fee_percent: u64,
        borrower_fee_percent: u64,
        max_offer_interest: u64,
        min_health_ratio: u64,
        hot_wallet: address,
    ) {
        configuration.lender_fee_percent = lender_fee_percent;
        configuration.borrower_fee_percent = borrower_fee_percent;
        configuration.max_offer_interest = max_offer_interest;
        configuration.min_health_ratio = min_health_ratio; 
        configuration.hot_wallet = hot_wallet;
    }

    public(package) fun add<K: copy + drop + store, V: store>(
        configuration: &mut Configuration,
        key: K,
        value: V
    ) {
        field::add(&mut configuration.id, key, value);
    }

    public(package) fun borrow<K: copy + drop + store, V: store>(
        configuration: &Configuration,
        key: K
    ): &V {
        field::borrow<K, V>(&configuration.id, key)
    }

    public(package) fun borrow_mut<K: copy + drop + store, V: store>(
        configuration: &mut Configuration,
        key: K
    ): &mut V {
        field::borrow_mut<K, V>(&mut configuration.id, key)
    }

    public(package) fun contain<K: copy + drop + store, V: store>(
        configuration: &Configuration,
        key: K
    ): bool {
        field::exists_with_type<K, V>(&configuration.id, key)
    }

    public(package) fun remove<K: copy + drop + store, V: store>(
        configuration: &mut Configuration,
        key: K
    ): V {
        field::remove<K, V>(&mut configuration.id, key)
    }

    public fun lender_fee_percent(
        configuration: &Configuration
    ): u64 {
        configuration.lender_fee_percent
    }

    public fun borrower_fee_percent(
        configuration: &Configuration
    ): u64 {
        configuration.borrower_fee_percent
    }

    public fun max_offer_interest(
        configuration: &Configuration
    ): u64 {
        configuration.max_offer_interest
    }

    public fun hot_wallet(
        configuration: &Configuration
    ): address {
        configuration.hot_wallet
    }

    public fun min_health_ratio(
        configuration: &Configuration
    ): u64 {
        configuration.min_health_ratio
    }
}