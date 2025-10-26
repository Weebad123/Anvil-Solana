use anchor_lang::prelude::*;

use crate::{CollateralVaultError, ModifyCollateralizableTokenAllowance};


pub fn modify_collateralizable_token_allowance(
    ctx: Context<ModifyCollateralizableTokenAllowance>, by_amount: i64
) -> Result<()> {

    require!(by_amount > 0, CollateralVaultError::ZeroAmount);

    ctx.accounts.authorized_modify_collateralizable_token_allowance(by_amount)?;
    
    Ok(())
}