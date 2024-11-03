module enso_lending::version {

    const EVersionMismatch: u64 = 601;

    const CURRENT_VERSION: u64 = 1;

    public struct Version has key, store {
        id: UID,
        value: u64,
    }

    public struct VersionCap has key, store {
        id: UID
    }

    fun init(ctx: &mut TxContext) {
        let version = Version {
            id: object::new(ctx),
            value: CURRENT_VERSION,
        };
        let cap = VersionCap {
            id: object::new(ctx),
        };
        transfer::share_object(version);
        transfer::transfer(cap, tx_context::sender(ctx));
    }

    public entry fun upgrade(_: &VersionCap, v: &mut Version) {
        v.value = CURRENT_VERSION + 1;
    }

    public fun value(v: &Version): u64 { v.value }

    public fun is_current_version(v: &Version): bool {
        v.value == CURRENT_VERSION
    }

    public fun assert_current_version(v: &Version) {
        assert!(is_current_version(v), EVersionMismatch);
    }
}