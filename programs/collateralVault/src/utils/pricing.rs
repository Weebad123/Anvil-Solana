//use anchor_lang::solana_program::example_mocks::solana_rpc_client_nonce_utils::Error;
use anchor_lang::prelude::*;

pub struct OraclePrice {

    pub price: u128,

    pub exponent: i32,

    pub publish_time: u128,
}


pub fn percentage_of(amount: u128, percentage_basis_points: u128) -> Result<u128> {

    let result = amount.checked_mul(percentage_basis_points).unwrap();

    let finalized = result.checked_div(10_000).unwrap();

    Ok(finalized)
}

pub fn amount_with_fee(amount: u128, fee_basis_points: u128) -> Result<u128> {

    let final_value = amount.checked_add(percentage_of(amount, fee_basis_points)?)
        .unwrap();

    Ok(final_value)
}


pub fn amount_before_fee(amount: u128, fee_basis_points: u128) -> Result<u128> {

    let value = (amount.checked_mul(10_000).unwrap())   
        .checked_div((fee_basis_points.checked_add(10_000)).unwrap()).unwrap();

    Ok(value)
}