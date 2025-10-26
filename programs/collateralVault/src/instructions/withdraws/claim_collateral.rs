use anchor_lang::prelude::*;

use crate::ClaimCollateral;



pub fn claim_collateral_funds(ctx: Context<ClaimCollateral>, amount_to_receive: u64, 
    release_remainder: bool) -> Result<(u128, u128)> {

        let (remaining_reserved_collateral, remaining_claimable_collateral) = &mut ctx.accounts
            .initiate_claim(amount_to_receive, release_remainder)?;

        Ok((*remaining_reserved_collateral, *remaining_claimable_collateral))
    }