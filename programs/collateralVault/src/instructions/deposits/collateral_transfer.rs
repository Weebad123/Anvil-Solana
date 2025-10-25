use anchor_lang::prelude::*;

use crate::{CollateralUtils, TransferCollateral};


// FROM USER TO USER, OR CONTRACT TO CONTRACT, TO ANYBODY
pub fn transfer_collateral(ctx: Context<TransferCollateral>, amount: u128) -> Result<()> {

    ctx.accounts.transfer_collateral(amount)?;
    Ok(())
}