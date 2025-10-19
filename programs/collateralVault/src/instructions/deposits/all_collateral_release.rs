use anchor_lang::prelude::*;


//use crate::utils::pricing::amount_before_fee;

use crate::states::user_contexts::*/*AccountCollateralizableAllowance, AccountsBalance, */;


pub fn release_all_collateral(ctx: Context<ReleaseAllCollateral>, _reservation_id: u64) -> Result<u128> {

    let total_released_collateral = ctx.accounts.collateral_reservations.token_amount;

    let account_balance_storage = &mut ctx.accounts.account_balance_pda;

    account_balance_storage.collateral_balance.available = account_balance_storage
        .collateral_balance.available.checked_add(total_released_collateral as u64).unwrap();

    account_balance_storage.collateral_balance.reserved = account_balance_storage
        .collateral_balance.reserved.checked_sub(total_released_collateral as u64).unwrap();

    // Close Reservation Account  => Equivalent Solidity Delete
    ctx.accounts.collateral_reservations.close(ctx.accounts.reserving_contract.to_account_info())?;

    Ok(total_released_collateral)
}