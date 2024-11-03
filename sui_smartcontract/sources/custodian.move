module enso_lending::custodian {
    use sui::{balance::{Self, Balance}};

    public struct Custodian<phantom T> has key, store {
        id: UID,
        treasury_balance: Balance<T>,
    }
    
    public(package) fun new<T>(ctx: &mut TxContext) {
        let custodian = Custodian<T> {
            id: object::new(ctx),
            treasury_balance: balance::zero<T>(),
        };

        transfer::public_share_object(custodian);
    }

    public(package) fun add_treasury_balance<T>(
        custodian: &mut Custodian<T>,
        amount: Balance<T>,
    ) {
        custodian.treasury_balance.join<T>(amount);
    }

    public(package) fun sub_treasury_balance<T>(
        custodian: &mut Custodian<T>,
        amount: u64,
    ): Balance<T> {
        custodian.treasury_balance.split<T>(amount)
    }

    public fun treasury_balance<T>(
        custodian: &mut Custodian<T>,
    ): u64 {
        custodian.treasury_balance.value<T>()
    }
}