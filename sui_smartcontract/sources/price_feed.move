module enso_lending::price_feed {
    use sui::clock::Clock;
    use pyth::{
        price_info::PriceInfoObject,
        i64::I64,
        pyth::{get_price_no_older_than},
        price::Price,
    };
    use enso_lending::{
        asset::Asset,
        utils,
    };

    use fun i64_to_u64 as I64.to_u64;
    use fun is_negative as I64.is_negative;
    use fun enso_lending::utils::vector_to_hex_char as vector.to_hex_char;
    use fun pyth::price_info::get_price_info_from_price_info_object as PriceInfoObject.to_price_info;

    public fun get_price(
        price_info_object: &PriceInfoObject,
        time_threshold: u64,
        clock: &Clock,
    ): (u64, u64, bool) {
        let price: Price = get_price_no_older_than(price_info_object, clock, time_threshold);
        let price_u64 = price.get_price().to_u64();
        let exponent_u64 = price.get_expo().to_u64();
        let is_negative = price.get_expo().is_negative();

        (price_u64, exponent_u64, is_negative)
    }

    public fun get_value_by_usd<T>(
        price_info_object: &PriceInfoObject,
        max_decimals: u64,
        amount: u64,
        asset: &Asset<T>,
        clock: &Clock,
    ) : u128 {
        let coin_decimals_u8 = asset.decimals<T>();
        // Standardized amount to max decimals
        let amount_by_max_decimals = (amount as u128) * (utils::power(10, max_decimals) as u128) / (utils::power(10, (coin_decimals_u8 as u64)) as u128);

        let value_usd: u128;
        let (price_u64, exponent_u64, is_negative) = get_price(price_info_object, asset.max_price_age_seconds(), clock);
        let exponent_power = (utils::power(10, exponent_u64) as u128);

        if (is_negative) {
            value_usd = (price_u64 as u128) * amount_by_max_decimals / exponent_power;
        } else {
            value_usd = (price_u64 as u128) * amount_by_max_decimals * exponent_power;
        };

        value_usd
    }

    public fun is_valid_price_info_object<T>(
        price_info_object: &PriceInfoObject, 
        asset: &Asset<T>,
    ): bool {
        let price_info = price_info_object.to_price_info();
        let price_id = price_info.get_price_identifier().get_bytes();
        let price_feed_id = price_id.to_hex_char();

        price_feed_id == asset.price_feed_id()
    }

    public fun i64_to_u64(
        price: &I64,
    ): u64 {
        let price_value: u64;
        let is_negative = price.get_is_negative();
        if (!is_negative) {
            price_value = price.get_magnitude_if_positive();
        } else {
            price_value = price.get_magnitude_if_negative();
        };
        price_value
    }

    public fun is_negative (
        price: &I64,
    ): bool {
       price.get_is_negative()
    }
}