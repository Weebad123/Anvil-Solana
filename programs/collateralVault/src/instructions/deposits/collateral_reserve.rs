use anchor_lang::prelude::*;


use crate::utils::pricing::amount_before_fee;

use crate::{states::{errors::*, user_contexts::*, constants::*}, /*AccountCollateralizableAllowance, AccountsBalance, */
CollateralReservations, utils::CollateralUtils};


pub fn reserve_collateral<'info>(ctx: &mut Context<'_, '_, 'info, 'info, ReserveCollateral<'info>>, 
    account_address: Pubkey, amount: u64) -> Result<(u64, u64)> {

        // Verify Token Enabled
        let token_address = ctx.accounts.token_address.key();
        ctx.accounts.verify_token_enabled(token_address)?;

        let claimable_amount =  amount_before_fee(amount as u128, WITHDRAWAL_FEE_BASIS_POINTS as u128).unwrap() as u64;

        let reservation_id: u64 = reserve_collateral_internal(ctx, account_address, amount, claimable_amount)?;

        Ok((claimable_amount, reservation_id))
    }


pub fn reserve_collateral_internal<'info>(ctx: &mut Context<'_, '_, 'info, 'info, ReserveCollateral<'info>>,
    account_address: Pubkey, reserved_collateral: u64, claimable_collateral: u64) -> 
    Result<u64> {

        require!(claimable_collateral > 0, CollateralVaultError::ClaimableAmountZero);

        ctx.accounts.require_collateralizable_and_decrease_approved_amount(claimable_collateral as u128)?;

        let available_amount = ctx.accounts.account_balance_pda./*load()?.*/collateral_balance.available;
        require!( available_amount >= reserved_collateral, CollateralVaultError::InsufficientCollateral);

        // @dev sanity-check
        // UPDATE AVAILABLE AND RESERVED
        require!(reserved_collateral >= claimable_collateral, CollateralVaultError::InsufficientCollateral);
        ctx.accounts.account_balance_pda./*load_mut()?.*/collateral_balance.available = 
            ctx.accounts.account_balance_pda/*.load_mut()?*/.collateral_balance.available - reserved_collateral;
        ctx.accounts.account_balance_pda./*load_mut()?.*/collateral_balance.reserved = 
            ctx.accounts.account_balance_pda./*load_mut()?.*/collateral_balance.reserved + reserved_collateral;

        let withdrawal_fee = WITHDRAWAL_FEE_BASIS_POINTS as u16;

        // Set Reservation ID for This Reservation and Update Counter
        let reservation_id = ctx.accounts.collateral_reservations_nonce.nonce + 1;
        ctx.accounts.collateral_reservations_nonce.nonce = 
            ctx.accounts.collateral_reservations_nonce.nonce + 1;

        // Set Collateral Reservations
        ctx.accounts.collateral_reservations.set_inner( CollateralReservations {
            reserving_contract: *ctx.accounts.reserving_contract.key,
            account_address,
            token_address: ctx.accounts.token_address.key(),
            withdrawal_fee,
            token_amount: reserved_collateral as u128,
            claimable_collateral: claimable_collateral as u128,
        });

        Ok(reservation_id)
    }

