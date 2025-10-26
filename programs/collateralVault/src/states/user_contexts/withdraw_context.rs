use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::{WITHDRAWAL_FEE_BASIS_POINTS, states::{accounts::*, errors::*}, utils::{percentage_of, amount_with_fee}};

#[derive(Accounts)]
pub struct Withdraw<'info> {

    #[account(mut)]
    pub withdrawer: Signer<'info>,

    #[account()]
    pub token_address: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::authority = withdrawer.key(),
        associated_token::mint = token_address
    )]
    pub withdrawer_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"supported_token_registry"],
        bump = tokens_registry.token_registry_bump
    )]
    pub tokens_registry: Account<'info, TokenRegistry>,

     #[account(
        mut,
        seeds = [b"account_balance_pda", withdrawer.key().as_ref(), token_address.key().as_ref()],
        bump
    )]
    pub account_balance_pda: Account<'info, AccountsBalance>,

    /// CHECKED: Safe
    pub bank_token_vault: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::authority = bank_token_vault.key(),
        associated_token::mint = token_address
    )]
    pub bank_token_vault_ata: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Interface<'info, TokenInterface>
}

impl<'info> Withdraw<'info> {

    fn validate_withdraw(&mut self, amount: u128) -> Result<()> {

        require!(amount > 0, CollateralVaultError::ZeroAmount);
        let account_balance_withdrawer = &self.account_balance_pda;
        let available_withdrawable_amount = account_balance_withdrawer.collateral_balance.available as u128;

        require!(amount >= available_withdrawable_amount, CollateralVaultError::InsufficientCollateral);
    
        let vault_balance = self.bank_token_vault_ata.amount as u128;
        require!(vault_balance >= amount, CollateralVaultError::InsufficientVaultBalance);
        Ok(())
    }

    pub fn initiate_withdraw(&mut self, amount: u128) -> Result<()> {

        self.validate_withdraw(amount)?;

        let withdraw_fee = percentage_of(amount, 
        WITHDRAWAL_FEE_BASIS_POINTS as u128)?
        ;
        let accounts_for_transfer = TransferChecked {
            from: self.bank_token_vault_ata.to_account_info(),
            to: self.withdrawer_ata.to_account_info(),
            mint: self.token_address.to_account_info(),
            authority: self.bank_token_vault.to_account_info()
        };
        let token_program = self.token_program.to_account_info();
        let transfer_cpi = CpiContext::new(
            token_program, accounts_for_transfer
        );

        let final_amount = amount.checked_sub(withdraw_fee).unwrap();
        transfer_checked(transfer_cpi, final_amount.try_into().unwrap(), self.token_address.decimals)?;


        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(reservation_id: u64)]
pub struct ClaimCollateral<'info> {

    #[account(
        mut,
        constraint = collateralizable_contract.key() == collateral_reservations.reserving_contract
        @ CollateralVaultError::UnauthorizedCollateralizableContract
    )]
    pub collateralizable_contract: Signer<'info>,

    #[account(mut)]
    pub destination_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"collateral_reservations", reservation_id.to_le_bytes().as_ref()],
        bump
    )]
    pub collateral_reservations: Account<'info, CollateralReservations>,

    #[account(
        constraint = token_address.key() == collateral_reservations.token_address
        @ CollateralVaultError::TokenNotSupported
    )]
    pub token_address: InterfaceAccount<'info, Mint>,

    /// CHECK: SAFE as constraint applied
    #[account(
        mut,
        constraint = account_address.key() == collateral_reservations.account_address
        @ CollateralVaultError::WrongAccountAddress
    )]
    pub account_address: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"supported_token_registry"],
        bump = tokens_registry.token_registry_bump
    )]
    pub tokens_registry: Account<'info, TokenRegistry>,

     #[account(
        mut,
        seeds = [b"account_balance_pda", account_address.key().as_ref(), token_address.key().as_ref()],
        bump
    )]
    pub account_balance_pda: Account<'info, AccountsBalance>,

    /// CHECK: SAFE to use as validated off-chain
    pub bank_token_vault: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::authority = bank_token_vault.key(),
        associated_token::mint = token_address
    )]
    pub bank_token_vault_ata: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Interface<'info, TokenInterface>

}

impl<'info> ClaimCollateral<'info> {

   
    fn validate_claim_collateral(&self, amount_to_receive: u64) -> Result<()> {

        require!(amount_to_receive > 0, CollateralVaultError::ClaimAmountZero);
        let claimable_token_amount = self.collateral_reservations.claimable_collateral as u64;
        require!(claimable_token_amount >= amount_to_receive, CollateralVaultError::InsufficientCollateral);
        Ok(())
    }

    fn transfer_tokens(&self, amount: u128) -> Result<()> {

        let accounts_for_transfer = TransferChecked {
            from: self.bank_token_vault_ata.to_account_info(),
            to: self.destination_ata.to_account_info(),
            mint: self.token_address.to_account_info(),
            authority: self.bank_token_vault.to_account_info(),
        };

        let transfer_program = self.token_program.to_account_info();

        let transfer_cpi = CpiContext::new(transfer_program, 
        accounts_for_transfer);
        transfer_checked(transfer_cpi, amount.try_into()?, self.token_address.decimals)?;
        Ok(())
    }

    pub fn initiate_claim(&mut self, amount_to_receive: u64, mut release_remainder: bool) -> Result<(u128, u128)> {

        self.validate_claim_collateral(amount_to_receive)?;

        let collateral_token_storage = &mut self.tokens_registry;
        let account_balance_pda_storage = &mut self.account_balance_pda;
        let collateral_reservations_storage = &mut self.collateral_reservations;

        let claim_amount_with_fee;
        let remaining_reserved_collateral;
        let token_amount = collateral_reservations_storage.token_amount;
        let claimable_token_amount = collateral_reservations_storage.claimable_collateral;

        let remaining_claimable_collateral = claimable_token_amount
            .checked_sub(amount_to_receive as u128).unwrap();

        if remaining_claimable_collateral == 0 {
            release_remainder = true;
            remaining_reserved_collateral = 0;
            claim_amount_with_fee = token_amount;
        } else {
            remaining_reserved_collateral = amount_with_fee(
                remaining_claimable_collateral,
                collateral_reservations_storage.withdrawal_fee as u128
            )?;
            claim_amount_with_fee = token_amount.checked_sub(remaining_reserved_collateral).unwrap();
        }

        // Update Cumulative User Deposits
        match collateral_token_storage.collateral_tokens
            .iter_mut()
            .find(|(token, _)| *token == self.token_address.key()) {
                // Update Cumulative User Balance Of The Collateral Token
                Some((_, collateral_token_info)) => {
                    collateral_token_info.user_cumulative_balance = collateral_token_info
                        .user_cumulative_balance.checked_sub(claim_amount_with_fee.try_into().unwrap())
                        .ok_or(CollateralVaultError::TokenOverflowError)?;
                },
                None => {}
            }

        if release_remainder {
            account_balance_pda_storage.collateral_balance.reserved = account_balance_pda_storage
                .collateral_balance.reserved.checked_sub(token_amount.try_into()?).unwrap();

            account_balance_pda_storage.collateral_balance.available = account_balance_pda_storage
                .collateral_balance.available.checked_add(remaining_reserved_collateral.try_into()?)
                .unwrap();

            // Close The Collateral Reservations
            collateral_reservations_storage.close(
                self.collateralizable_contract.to_account_info())?;
        } else {
            account_balance_pda_storage.collateral_balance.reserved = account_balance_pda_storage
                .collateral_balance.reserved.checked_sub(claim_amount_with_fee.try_into()?).unwrap();

            collateral_reservations_storage.claimable_collateral = remaining_claimable_collateral;
            collateral_reservations_storage.token_amount = remaining_reserved_collateral;
        }

        let _claim_fee = claim_amount_with_fee.checked_sub(amount_to_receive.into()).unwrap();

        // Initiate Transfer
        self.transfer_tokens(amount_to_receive.into())?;


        Ok((remaining_reserved_collateral, remaining_claimable_collateral))
    }
}