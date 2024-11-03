module enso_lending::offer_registry {
    use sui::event;
    use std::string::String;
    
    use fun std::string::utf8 as vector.to_string;

    const CREATED_STATUS: vector<u8> = b"Created";
    const CANCELLING_STATUS: vector<u8> = b"Cancelling";
    const CANCELLED_STATUS: vector<u8> = b"Cancelled";
    const LOANED_STATUS: vector<u8> = b"Loaned";

    public struct OfferKey<phantom LendCoinType> has store, copy, drop {
        offer_id: ID,
    } 

    public struct Offer<phantom LendCoinType> has key, store {
        id: UID,
        asset_tier: ID,
        amount: u64,
        duration: u64,
        interest: u64,
        status: String,
        lender: address,
    }

    public struct NewOfferEvent has copy, drop {
        offer_id: ID,
        asset_tier_id: ID,
        asset_tier_name: String,
        amount: u64,
        duration: u64,
        interest: u64,
        status: String,
        lender: address,
    }

    public struct RequestCancelOfferEvent has copy, drop {
        offer_id: ID,
        amount: u64,
        duration: u64,
        interest: u64,
        lender: address,
    }

    public struct CancelledOfferEvent has copy, drop {
        offer_id: ID,
        amount: u64,
        duration: u64,
        interest: u64,
        lender: address,
    }

    public struct EditedOfferEvent has copy, drop {
        offer_id: ID,
        amount: u64,
        duration: u64,
        interest: u64,
        lender: address,
    }

    
    public(package) fun new_offer<LendCoinType>(
        asset_tier: ID,
        asset_tier_name: String,
        lend_amount: u64,
        duration: u64,
        interest: u64,
        lender: address,
        ctx: &mut TxContext,
    ): Offer<LendCoinType> {
        let offer = Offer<LendCoinType> {
            id: object::new(ctx),
            asset_tier,
            amount: lend_amount,
            duration,
            interest,
            status: CREATED_STATUS.to_string(),
            lender,
        };

        event::emit(NewOfferEvent {
            offer_id: object::id(&offer),
            asset_tier_id: asset_tier,
            asset_tier_name,
            amount: lend_amount,
            duration,
            interest,
            status: CREATED_STATUS.to_string(),
            lender
        });

        offer
    }

    public(package) fun cancel_offer<LendCoinType>(
        offer: &mut Offer<LendCoinType>,
        sender: address,
    ) {
        offer.status = CANCELLING_STATUS.to_string();

        event::emit(RequestCancelOfferEvent {
            offer_id: object::id(offer),
            amount: offer.amount,
            duration: offer.duration,
            interest: offer.interest,
            lender: sender,
        });
    }

    public(package) fun edit_offer<LendCoinType>(
        offer: &mut Offer<LendCoinType>,
        interest: u64,
        sender: address,
    ) {
        offer.interest = interest;

        event::emit(EditedOfferEvent {
            offer_id: object::id(offer),
            amount: offer.amount,
            duration: offer.duration,
            interest: offer.interest,
            lender: sender,
        });
    }

    public(package) fun system_cancel_offer<LendCoinType>(
        offer: &mut Offer<LendCoinType>,
    ) {
        offer.status = CANCELLED_STATUS.to_string();

        event::emit(CancelledOfferEvent {
            offer_id: object::id(offer),
            amount: offer.amount,
            duration: offer.duration,
            interest: offer.interest,
            lender: offer.lender,
        });
    }

    public(package) fun take_loan<LendCoinType>(
        offer: &mut Offer<LendCoinType>,
    ) {
        offer.status = LOANED_STATUS.to_string();
    }

    public fun new_offer_key<LendCoinType>(
        offer_id: ID,
    ): OfferKey<LendCoinType> {
        OfferKey<LendCoinType> {
            offer_id,
        }
    }

    public fun is_created_status<LendCoinType>(
        offer: &Offer<LendCoinType>
    ): bool {
        offer.status == CREATED_STATUS.to_string()
    }

    public fun is_cancelling_status<LendCoinType>(
        offer: &Offer<LendCoinType>
    ): bool {
        offer.status == CANCELLING_STATUS.to_string()
    }

    public fun amount<LendCoinType>(
        offer: &Offer<LendCoinType>
    ): u64 {
        offer.amount
    }

    public fun interest<LendCoinType>(
        offer: &Offer<LendCoinType>
    ): u64 {
        offer.interest
    }

    public fun duration<LendCoinType>(
        offer: &Offer<LendCoinType>
    ): u64 {
        offer.duration
    }

    public fun lender<LendCoinType>(
        offer: &Offer<LendCoinType>
    ): address {
        offer.lender
    }

    public fun asset_tier<LendCoinType>(
        offer: &Offer<LendCoinType>
    ): ID {
        offer.asset_tier
    }
}