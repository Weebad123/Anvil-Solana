use anchor_lang::prelude::*;

use crate::{CollateralUtils, states::*};


pub fn pool_collateral(ctx: Context<PoolCollateral>, amount: u128) -> Result<()> {

    ctx.accounts.verify_token_enabled(*ctx.accounts.token_address.key)?;

    ctx.accounts.require_collateralizable_and_decrease_approved_amount(amount)?;

    ctx.accounts.transfer_collateral(amount)?;
    
    Ok(())
}
