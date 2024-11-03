module enso_lending::vaa_utils {
	use std::string::String;
	use enso_lending::utils::string_to_u64;

	const EMissingDataOnVaa: u64 = 1;
	const EInvalidDataOnVaa: u64 = 2;
	
	public fun parse_cancel_collateral_payload(
		payload_body: vector<String>
	): (String, String) {
		assert!(vector::length<String>(&payload_body) == 2, EMissingDataOnVaa);
		let offer_id = *vector::borrow<String>(&payload_body, 0);
		let lend_chain_borrower = *vector::borrow<String>(&payload_body, 1);
		(offer_id, lend_chain_borrower)
	}

	public fun parse_liquidate_collateral_payload(
		payload_body: vector<String>
	): (String, String, u64, u64) {
		assert!(vector::length<String>(&payload_body) == 4, EMissingDataOnVaa);
		let offer_id = *vector::borrow<String>(&payload_body, 0);
		let lend_chain_borrower = *vector::borrow<String>(&payload_body, 1);
		let liquidating_price = *vector::borrow<String>(&payload_body, 2);
		let liquidating_at = *vector::borrow<String>(&payload_body, 3);

		let mut liquidating_price_u64_opt = string_to_u64(liquidating_price);
		let mut liquidating_at_u64_opt = string_to_u64(liquidating_at);
		assert!(
			option::is_some<u64>(&liquidating_price_u64_opt) &&
			option::is_some<u64>(&liquidating_at_u64_opt), 
			EInvalidDataOnVaa
		);

		(
			offer_id,
			lend_chain_borrower,
			option::extract<u64>(&mut liquidating_price_u64_opt),
			option::extract<u64>(&mut liquidating_at_u64_opt),
		)
	}

	public fun parse_refund_collateral_to_repaid_borrower_payload(
		payload_body: vector<String>
	): (String, String) {
		assert!(vector::length<String>(&payload_body) == 2, EMissingDataOnVaa);
		let offer_id = *vector::borrow<String>(&payload_body, 0);
		let lend_chain_borrower = *vector::borrow<String>(&payload_body, 1);
		(offer_id, lend_chain_borrower)
	}
}