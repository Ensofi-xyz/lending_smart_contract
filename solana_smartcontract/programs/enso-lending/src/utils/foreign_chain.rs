use anchor_lang::prelude::*;
use crate::{WormholeError, SUI_CHAIN_ADDRESS, CHAIN_IDS};

pub fn get_chain_address_by_chain_id(chain_id: u16) -> Result<String> {
  for &id in &CHAIN_IDS {
    if id == chain_id {
      match chain_id {
        21 => return Ok(SUI_CHAIN_ADDRESS.to_string()),
        _ => {
          return Err(WormholeError::NotSupportThisChainId.into());
        },
      }
    }
  }
  return Err(WormholeError::NotSupportThisChainId.into());
}