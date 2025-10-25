use anchor_lang::prelude::*;

use crate::{ CollateralUtils, ModifyCollateralReservations, amount_before_fee, states::errors::*};


pub fn modify_collateral_reservations(ctx: Context<ModifyCollateralReservations>, 
        _reservation_id: u64, by_amount: i128) -> Result<(u128, u128)> {

            
            let old_reserved_amount = ctx.accounts.collateral_reservations.token_amount;
            require!(old_reserved_amount > 0, CollateralVaultError::CollateralReservationNotFound);

            if by_amount == 0 {
                return Ok((ctx.accounts.collateral_reservations.token_amount, 
                ctx.accounts.collateral_reservations.claimable_collateral))
            }

            let reserved_collateral: u128;
            let claimable_collateral: u128;
            
            if by_amount < 0 {
                let by_amount_positive = by_amount.checked_mul(-1).unwrap() as u128;
                require!(by_amount_positive < old_reserved_amount, CollateralVaultError::InsufficientCollateral);

                let collateral_reservations_storage = &mut ctx.accounts.collateral_reservations;
                let collateral_balances_storage = &mut ctx.accounts.account_balance_pda;

                reserved_collateral = old_reserved_amount.checked_sub(by_amount_positive).unwrap();
                collateral_reservations_storage.token_amount = reserved_collateral;

                // Increase Available, Decrease Reserve on Account Balance
                collateral_balances_storage.collateral_balance.available = collateral_balances_storage
                    .collateral_balance.available.checked_add(by_amount_positive as u64).unwrap();
                collateral_balances_storage.collateral_balance.reserved = collateral_balances_storage
                    .collateral_balance.reserved.checked_sub(by_amount_positive as u64).unwrap();
            } else {
                let by_amount_positive = by_amount as u128;
                
                ctx.accounts.require_collateralizable_and_decrease_approved_amount(by_amount_positive)?;

                let collateral_reservations_storage = &mut ctx.accounts.collateral_reservations;
                let collateral_balances_storage = &mut ctx.accounts.account_balance_pda;

                let available = collateral_balances_storage.collateral_balance.available as u128;
                require!(available > by_amount_positive, CollateralVaultError::InsufficientCollateral);

                reserved_collateral = old_reserved_amount.checked_add(by_amount_positive).unwrap();
                collateral_reservations_storage.token_amount = reserved_collateral;

                // Increase Reserve, Decrease Available On Account Balance
                collateral_balances_storage.collateral_balance.available = collateral_balances_storage
                    .collateral_balance.available.checked_sub(by_amount_positive as u64).unwrap();
                collateral_balances_storage.collateral_balance.reserved = collateral_balances_storage
                    .collateral_balance.reserved.checked_add(by_amount_positive as u64).unwrap();
            }

            let collateral_reservations_storage = &mut ctx.accounts.collateral_reservations;
            claimable_collateral = amount_before_fee(reserved_collateral, collateral_reservations_storage.withdrawal_fee as u128)?;
            require!(claimable_collateral > 0, CollateralVaultError::ClaimableAmountZero);
            
            collateral_reservations_storage.claimable_collateral = claimable_collateral;
            
            Ok((reserved_collateral, claimable_collateral))
        }