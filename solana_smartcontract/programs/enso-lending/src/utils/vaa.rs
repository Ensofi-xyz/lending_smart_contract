use solana_program::msg;

use crate::{ParseVaaError, HEX_MIN_WIDTH};

pub fn parse_create_loan_payload(
    posted_vaa: &Vec<u8>,
) -> Result<(u16, String, String, String, String, u64, u64, String, String), ParseVaaError> {
  let message = String::from_utf8_lossy(posted_vaa).into_owned();
  let data: Vec<&str> = message.split(',').collect();
  msg!("Message received: {:?}", data);

  let target_chain = data[0].parse::<u16>().map_err(|_| ParseVaaError::InvalidTargetChain)?;
  let target_address = data[1].to_string();
  let target_function = data[2].to_string();
  let tier_id = data[3].to_string();
  let offer_id = data[4].to_string();
  let lend_amount = data[5].parse::<u64>().map_err(|_| ParseVaaError::InvalidLendAmount)?;
  let collateral_amount = data[6].parse::<u64>().map_err(|_| ParseVaaError::InvalidCollateralAmount)?;
  let collateral_address = data[7].to_string();
  let borrower_address = data[8].to_string();

  Ok((
    target_chain,
    target_address,
    target_function,
    tier_id,
    offer_id,
    lend_amount,
    collateral_amount,
    collateral_address,
    borrower_address
  ))
}

pub fn validate_posted_vaa(
    posted_chain_id: u16,
    chain_id: u16,
    posted_emitter_address: [u8; 32],
    emitter_address: String,
) -> Result<(), ParseVaaError> {
    if posted_chain_id == chain_id {
        let posted_emitter_address = posted_emitter_address
            .iter()
            .map(|&c| {
                if c < HEX_MIN_WIDTH {
                    format!("0{:x}", c)
                } else {
                    format!("{:x}", c)
                }
            })
            .collect::<String>();

        if posted_emitter_address != emitter_address {
            return Err(ParseVaaError::InvalidForeignEmitter);
        }
    } else {
        return Err(ParseVaaError::InvalidTargetChain);
    }

    Ok(())
}

pub fn parse_deposit_collateral_payload(
    posted_vaa: &Vec<u8>,
) -> Result<(u16, String, String, String, u64, String, String), ParseVaaError> {
  let message = String::from_utf8_lossy(posted_vaa).into_owned();
  let data: Vec<&str> = message.split(',').collect();
  msg!("Message received: {:?}", data);

  let target_chain = data[0].parse::<u16>().map_err(|_| ParseVaaError::InvalidTargetChain)?;
  let target_address = data[1].to_string();
  let target_function = data[2].to_string();
  let loan_offer_id = data[3].to_string();
  let collateral_amount = data[4].parse::<u64>().map_err(|_| ParseVaaError::InvalidCollateralAmount)?;
  let collateral_address = data[5].to_string();
  let borrower_address = data[6].to_string();

  Ok((
    target_chain,
    target_address,
    target_function,
    loan_offer_id,
    collateral_amount,
    collateral_address,
    borrower_address
  ))
}

pub fn parse_withdraw_collateral_payload(
    posted_vaa: &Vec<u8>,
) -> Result<(u16, String, String, String, u64, u64, String, String), ParseVaaError> {
  let message = String::from_utf8_lossy(posted_vaa).into_owned();
  let data: Vec<&str> = message.split(',').collect();
  msg!("Message received: {:?}", data);

  let target_chain = data[0].parse::<u16>().map_err(|_| ParseVaaError::InvalidTargetChain)?;
  let target_address = data[1].to_string();
  let target_function = data[2].to_string();
  let loan_offer_id = data[3].to_string();
  let withdraw_amount = data[4].parse::<u64>().map_err(|_| ParseVaaError::InvalidCollateralAmount)?;
  let remaining_collateral_amount = data[5].parse::<u64>().map_err(|_| ParseVaaError::InvalidRemainingCollateralAmount)?;
  let collateral_address = data[6].to_string();
  let borrower_address = data[7].to_string();

  Ok((
    target_chain,
    target_address,
    target_function,
    loan_offer_id,
    withdraw_amount,
    remaining_collateral_amount,
    collateral_address,
    borrower_address
  ))
}