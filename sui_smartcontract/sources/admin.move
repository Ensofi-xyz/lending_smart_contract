module enso_lending::admin {
    use sui::balance::Balance;

    use enso_lending::{
        version::Version,
        operator,
        custodian::Custodian,
    };

    use fun sui::coin::from_balance as Balance.into_coin;

    public struct AdminCap has key, store {
        id: UID,
    }

    fun init(ctx: &mut TxContext) {
        let admin_cap = AdminCap {
            id: object::new(ctx),
        };

        transfer::transfer(admin_cap, @admin);
    }

    public entry fun set_admin(
        _: &AdminCap,
        version: &Version,
        user_address: address,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();
        let admin_cap = AdminCap {
            id: object::new(ctx),
        };

        transfer::transfer(admin_cap, user_address);
    }

    public entry fun set_operator(
        _: &AdminCap,
        version: &Version,
        user_address: address,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();
        operator::new_operator(user_address, ctx);
    }

    public entry fun claim_treasury_balance<T>(
        _: &AdminCap,
        version: &Version,
        custodian: &mut Custodian<T>,
        ctx: &mut TxContext,
    ) {
        version.assert_current_version();
        let amount = custodian.treasury_balance();
        let balance = custodian.sub_treasury_balance(amount);
        transfer::public_transfer(balance.into_coin(ctx), ctx.sender());
    }
}