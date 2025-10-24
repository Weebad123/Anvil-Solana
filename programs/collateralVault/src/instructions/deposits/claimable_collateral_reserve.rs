use anchor_lang::prelude::*;

use crate::{states::{user_contexts::*, constants::*}, utils::pricing::amount_with_fee};
use crate::instructions::reserve_collateral_internal;


pub fn reserve_claimable_collateral<'info>(ctx: &mut Context<'_, '_, 'info, 'info, ReserveCollateral<'info>>, account_address: Pubkey,
    claimable_amount: u128) -> Result<(u64, u64)> {

        let total_amount_reserved = amount_with_fee(
                claimable_amount, WITHDRAWAL_FEE_BASIS_POINTS as u128).unwrap() as u64;

        let reservation_id = reserve_collateral_internal(
            ctx, account_address, total_amount_reserved, claimable_amount as u64
        )?;

        Ok((total_amount_reserved, reservation_id))
    }