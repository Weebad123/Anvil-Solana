use anchor_lang::prelude::*;

use crate::Withdraw;
use crate::states::errors::*;


pub fn withdraw(ctx: Context<Withdraw>, amount: u128) -> Result<()> {

    ctx.accounts.initiate_withdraw(amount)?;

    let accounts_balance = &mut ctx.accounts.account_balance_pda;
    let collateral_token_storage = &mut ctx.accounts.tokens_registry;

    accounts_balance.collateral_balance.available = accounts_balance.collateral_balance.available
        .checked_sub(amount as u64)
        .unwrap();
    
    // Update Cumulative User Deposits
     match collateral_token_storage.collateral_tokens
        .iter_mut()
        .find(|(token, _)| *token == ctx.accounts.token_address.key()) {
            // Update Cumulative User Balance Of The Collateral Token
            Some((_, collateral_token_info)) => {
                collateral_token_info.user_cumulative_balance = collateral_token_info
                    .user_cumulative_balance.checked_sub(amount.try_into().unwrap())
                    .ok_or(CollateralVaultError::TokenOverflowError)?;
            },
            None => {}
        }
    Ok(())
}