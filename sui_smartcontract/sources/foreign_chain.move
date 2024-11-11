module enso_lending::foreign_chain {
  
    public struct ForeignChainKey has store, copy, drop {
        chain_id: u16,
    }

    public struct ForeignChain has store, drop {
        chain_address: vector<u8>,
        emitter_address: vector<u8>, //buffer
    }

	public(package) fun new_foreign_chain(
		chain_address: vector<u8>,
		emitter_address: vector<u8>,
	): ForeignChain {
		ForeignChain {
			chain_address,
			emitter_address
		}
	}

	public(package) fun update_foreign_chain(
		foreign_chain: &mut ForeignChain,
		chain_address: vector<u8>,
		emitter_address: vector<u8>,
	) {
		foreign_chain.chain_address = chain_address;
		foreign_chain.emitter_address = emitter_address;
	}

	public fun new_foreign_chain_key(chain_id: u16): ForeignChainKey {
		ForeignChainKey {
			chain_id
		}
	}

	public fun chain_address(foreign_chain: &ForeignChain): vector<u8> {
		foreign_chain.chain_address
	}

	public fun emitter_address(foreign_chain: &ForeignChain): vector<u8> {
		foreign_chain.emitter_address
	}
}