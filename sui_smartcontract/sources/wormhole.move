module enso_lending::wormhole {
	use std::string::String;
	use sui::{
		sui::SUI,
		coin::Coin,
		clock::Clock
	};
    use wormhole::{
		emitter::{Self, EmitterCap},
		state::State,
		publish_message,
		vaa,
	};
	use enso_lending::utils::convert_vaa_buffer_to_string;

	use fun std::string::utf8 as vector.to_string;

	const EInvalidVaa: u64 = 1;

	const CHAIN_ID: vector<u8> = b"21";

	public struct WORMHOLE has drop {}

	public struct ProtectedEC has key, store {
		id: UID,
		emitter_cap: EmitterCap,
	}

	public(package) fun new(
		wormhole_state: &State,
		ctx: &mut TxContext
	) {
		let emitter_cap = emitter::new(wormhole_state, ctx); 
        let protected_ec = ProtectedEC {
            id: object::new(ctx),
            emitter_cap,
        };
        transfer::public_share_object(protected_ec);
	}

	public(package) fun send_message(
        protected_ec: &mut ProtectedEC,
        wormhole_state: &mut State,
        payload: vector<u8>,
        message_fee: Coin<SUI>,
        clock: &Clock,
    ): u64 {
        let emitter_cap = &mut protected_ec.emitter_cap;
        let message = publish_message::prepare_message(
            emitter_cap,
            0,
            payload
        );

        let sequence = publish_message::publish_message(
            wormhole_state,
            message_fee,
            message,
            clock,
        );

        sequence
    }

	public(package) fun parse_and_verify_vaa(
		wormhole_state: &State,
		vaa_buf: vector<u8>,
		expected_emitter_chain: u16,
		expected_emitter_address: vector<u8>,
		expected_target_function: vector<u8>,
		clock: &Clock,
	): vector<String> {
		let vaa = vaa::parse_and_verify(
			wormhole_state,
			vaa_buf,
			clock,
		);
		let ( emitter_chain, emitter_address, vaa_payload ) = vaa.take_emitter_info_and_payload();
		let mut payload_data = convert_vaa_buffer_to_string(vaa_payload);
		let target_chain = *vector::borrow<String>(&payload_data, 0);
		let target_function = *vector::borrow<String>(&payload_data, 2);
		assert!(
			expected_emitter_chain == emitter_chain &&
			expected_emitter_address == emitter_address.to_bytes() &&
			target_chain == CHAIN_ID.to_string() /* SUI */ &&
			expected_target_function.to_string() == target_function,
			EInvalidVaa,
		);

		let mut i = vector::length(&payload_data) - 1;
		let mut payload_body: vector<String> = vector[];
		while(i > 2) {
			vector::push_back<String>(&mut payload_body, vector::pop_back(&mut payload_data));
			i = i - 1;
		}; 
		vector::reverse<String>(&mut payload_body);

		payload_body
	}
}