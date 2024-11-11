module enso_lending::operator_offer {
    use sui::coin::{Self, Coin};
    use enso_lending::{
        version::{Version},
        state::State,
        offer_registry::{Self, OfferKey, Offer},
        operator::OperatorCap,
    };

    const ENotFoundOfferToCancel: u64 = 1;
    const EInvalidOfferStatus: u64 = 2;
    const ELendCoinIsInvalid: u64 = 3;

    entry fun system_cancel_offer<LendCoinType>(
        _: &OperatorCap,
        version: &Version,
        state: &mut State,
        offer_id: ID,
        lend_coin: Coin<LendCoinType>,
        waiting_interest: Coin<LendCoinType>,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();

        let offer_key = offer_registry::new_offer_key<LendCoinType>(offer_id);
        assert!(state.contain<OfferKey<LendCoinType>, Offer<LendCoinType>>(offer_key), ENotFoundOfferToCancel);
        let offer = state.borrow_mut<OfferKey<LendCoinType>, Offer<LendCoinType>>(offer_key);

        assert!(offer.is_cancelling_status<LendCoinType>(), EInvalidOfferStatus);
        assert!(lend_coin.value() == offer.amount(), ELendCoinIsInvalid);

        let mut refund_coin = coin::zero<LendCoinType>(ctx);
        refund_coin.join<LendCoinType>(lend_coin);
        refund_coin.join<LendCoinType>(waiting_interest);
        transfer::public_transfer(refund_coin, offer.lender());

        offer.system_cancel_offer();
    }
}