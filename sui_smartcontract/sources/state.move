module enso_lending::state {
    use sui::{
        dynamic_object_field as ofield,
        table::{Self, Table},
    };

    public struct State has key, store{
        id: UID,
        offers: Table<address, Table<u64, ID>>, 
        loans: Table<address, Table<u64, ID>>,
    }

    public(package) fun new(
        ctx: &mut TxContext,
    ) {
        let state = State {
            id: object::new(ctx),
            offers: table::new<address, Table<u64, ID>>(ctx),
            loans: table::new<address, Table<u64, ID>>(ctx),
        };

        transfer::public_share_object(state);
    }
    
    public(package) fun add_offer(
        state: &mut State,
        offer_id: ID,
        lender: address,
        ctx: &mut TxContext,
    ) {
        if (!state.offers.contains<address, Table<u64, ID>>(lender)) {
            state.offers.add<address, Table<u64, ID>>(lender, table::new<u64,ID>(ctx));
        };
        let user_offers = state.offers.borrow_mut<address, Table<u64, ID>>(lender);
        let length = user_offers.length<u64, ID>();

        user_offers.add<u64, ID>(length + 1, offer_id);
    }

    public(package) fun add_loan(
        state: &mut State,
        loan_id: ID,
        borrower: address,
        ctx: &mut TxContext,
    ) {
        if (!state.loans.contains<address, Table<u64, ID>>(borrower)) {
            state.loans.add<address, Table<u64, ID>>(borrower, table::new<u64,ID>(ctx));
        };
        let user_loans = state.loans.borrow_mut<address, Table<u64, ID>>(borrower);
        let length = user_loans.length<u64, ID>();

       user_loans.add<u64, ID>(length + 1, loan_id);
    }

    public(package) fun add<K: copy + drop + store, V: key + store>(
        state: &mut State,
        key: K,
        value: V
    ) {
        ofield::add(&mut state.id, key, value);
    }

    public(package) fun borrow<K: copy + drop + store, V: key + store>(
        state: &State,
        key: K
    ): &V {
        ofield::borrow<K, V>(&state.id, key)
    }

    public(package) fun borrow_mut<K: copy + drop + store, V: key + store>(
        state: &mut State,
        key: K
    ): &mut V {
        ofield::borrow_mut<K, V>(&mut state.id, key)
    }

    public(package) fun contain<K: copy + drop + store, V: key + store>(
        state: &State,
        key: K
    ): bool {
        ofield::exists_with_type<K, V>(&state.id, key)
    }

    public(package) fun remove<K: copy + drop + store, V: key + store>(
        state: &mut State,
        key: K
    ): V {
        ofield::remove<K, V>(&mut state.id, key)
    }
}