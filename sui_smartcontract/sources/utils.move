module enso_lending::utils {
    use std::{
        type_name,
        string::{Self, String},
        ascii::{Self, String as AsciiString},
    };

    use fun std::string::utf8 as vector.to_string;
    use fun std::string::from_ascii as AsciiString.to_string; 

    const HEXTABLE : vector<vector<u8>> = vector[b"0", b"1", b"2", b"3", b"4", b"5", b"6", b"7", b"8", b"9", b"a", b"b", b"c", b"d", b"e", b"f"];
    const DEFAULT_RATE_FACTOR: u64 = 10000;
    const SECOND_IN_YEAR: u64 = 31536000;

    public fun power(base: u64, exponent: u64): u64 {
        let mut result = 1;
        let mut i = 1;
            
        while (i <= exponent) {
            result = result * base;
            i = i + 1;
        };
        
        result
    }

    public fun vector_to_hex_char (
        decimal: vector<u8>,
    ): String {
        let mut i = 0;
        let mut hex_string = string::utf8(b"0x");
        while (i < vector::length<u8>(&decimal)) {
            let element = vector::borrow<u8>(&decimal, i);
            let quotient = *element / 16;
            let rest = *element % 16;
            let quotient_to_hex = decimal_to_hex_char(quotient);
            let rest_to_hex = decimal_to_hex_char(rest);

            hex_string.append(quotient_to_hex);
            hex_string.append(rest_to_hex);
            
            i = i + 1;
        };

        hex_string
    }

    #[allow(implicit_const_copy)]
    public fun decimal_to_hex_char(
        element: u8,
    ): String {
        let value = *vector::borrow<vector<u8>>(&HEXTABLE, (element as u64));
            
        value.to_string()
    }

    public fun u64_to_string(num: u64): vector<u8> {
        let mut result = vector::empty<u8>();
        let mut temp = num;
        let zero_ascii = 48;

        if (temp == 0) {
            vector::push_back(&mut result, zero_ascii);
            return result
        };

        while (temp > 0) {
            let digit = ((temp % 10) as u8);
            vector::push_back(&mut result, zero_ascii + digit);
            temp = temp / 10;
        };

        vector::reverse(&mut result);

        result
    }

    public fun string_to_u64(input: String): Option<u64> {
        let bytes = string::bytes(&input);
        let mut result = 0u64;

        let len = vector::length(bytes);
        let mut i = 0;
        while (i < len) {
            let c = *vector::borrow(bytes, i);
            if (!is_digit(c)) {
                return option::none<u64>()
            };

            let digit = to_digit(c);
            result = result * 10 + digit;
            i = i + 1;
        };
        option::some(result)
    }

    public fun convert_vaa_buffer_to_string(buffer: vector<u8>): vector<String> {
        let mut result: vector<String> = vector[];
        let mut ascii_string = ascii::string(b"");

        let length = vector::length(&buffer);
        let mut i = 0;
        while (i < length) {
            let byte = *vector::borrow(&buffer, i);
            // comma
            if (byte == 44) {
                vector::push_back<String>(&mut result, ascii_string.to_string());
                ascii_string = ascii::string(b"");
            } else {
                let char = ascii::char(byte);
                ascii_string.push_char(char);
            };
            i = i + 1;
        };
        
        //push the last string 
        vector::push_back<String>(&mut result, ascii_string.to_string());

        result
    }

    public fun generate_message_vaa_payload(data: vector<vector<u8>>): vector<u8> {
        let mut payload: vector<u8> = vector[];
        let mut i = 0;
        while (i < vector::length(&data)) {
            vector::append(&mut payload, *vector::borrow(&data, i));
            vector::append(&mut payload, b",");
            i = i + 1;
        };

        payload
    }

    public fun get_type<T>(): String {
        let type_name = type_name::get<T>();
        string::from_ascii(type_name.into_string())
    }

    public fun default_rate_factor(): u64 {
        DEFAULT_RATE_FACTOR
    }

    public fun seconds_in_year(): u64 {
        SECOND_IN_YEAR
    }

    fun is_digit(char: u8): bool {
        // ASCII values for '0' to '9' are 48 to 57
        char >= 48 && char <= 57
    }

    fun to_digit(char: u8): u64 {
        // ASCII value of '0' is 48
        (char - 48) as u64
    }
}
