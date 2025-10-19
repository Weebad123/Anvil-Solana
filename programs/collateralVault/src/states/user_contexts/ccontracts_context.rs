use anchor_lang::prelude::*;

use anchor_spl::token_interface::Mint;

use crate::states::{ errors::*, accounts::*};

#[derive(Accounts)]
//#[instruction(collateralizable_contract_address: Pubkey)]
pub struct ReserveCollateral<'info> {
    #[account(mut)]
    /// CHECK: Safe To Use
    pub account_address: AccountInfo<'info>,

    #[account(mut)]
    pub reserving_contract: Signer<'info>,

    #[account(mut)]
    pub token_address: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"supported_token_registry"],
        bump = tokens_registry.token_registry_bump
    )]
    pub tokens_registry: Account<'info, TokenRegistry>,

    #[account(
        mut,
        seeds = [b"collateralizable_contracts"],
        bump = collateralizable_contracts.collaterizable_contracts_bump,
    )]
    pub collateralizable_contracts: Account<'info, CollateralizableContracts>,


    #[account(
        mut,
        seeds = [b"account_balance_pda", account_address.key().as_ref(), token_address.key().as_ref()],
        bump
    )]
    pub account_balance_pda: Account/*Loader*/<'info, AccountsBalance>,

     #[account(
        mut,
        seeds = [account_address.key().as_ref(), reserving_contract.key().as_ref(), token_address.key().as_ref()],
        bump
    )]
    pub account_collateralizable_allowance: Account/*Loader*/<'info, AccountCollateralizableAllowance>,

    #[account(
        mut,
        seeds = ["collateral_reservations_nonce".as_ref()],
        bump,
    )]
    pub collateral_reservations_nonce: Account<'info, CollateralReservationsNonce>,

    #[account(
        init,
        payer = reserving_contract,
        space = 8 + CollateralReservations::INIT_SPACE,
        seeds = [b"collateral_reservations".as_ref(), (collateral_reservations_nonce.nonce + 1).to_le_bytes().as_ref()],
        bump
    )]
    pub collateral_reservations: Account<'info, CollateralReservations>,

    pub system_program: Program<'info, System>,

}

impl<'info> ReserveCollateral<'info> {
    // Verify That The Given token Address Is Allowed By The Only Owner.
    pub fn verify_token_enabled(&self, token_address: Pubkey) -> Result<()> {

        let token_registry = &self.tokens_registry;

        // Check Through The Token Registry To Ensure That Indeed The Provided Token Address Is Allowed
        let tokens_info = token_registry.collateral_tokens
            .iter()
            .find(|(address, _)| address == &token_address);

        match tokens_info {
            Some((_, token)) => {
                require!(token.is_enabled, CollateralVaultError::TokenNotAllowed);
            },
            None => {
                return err!(CollateralVaultError::TokenNotSupported);
            }
        }
        Ok(())
    }

    pub fn require_collateralizable_and_decrease_approved_amount(&mut self, amount: u128) -> Result<()> {
        //@TODO should Just Return, Not Revert
        require!(self.account_address.key() != self.reserving_contract.key(), 
        CollateralVaultError::SameAsReservingContract);

        require!(self.collateralizable_contracts.collaterizable_contracts.contains(&self.reserving_contract.key()),
        CollateralVaultError::UnapprovedCollateralizableContract);

        let approved_amount: u128 = self.account_collateralizable_allowance./*load()?.*/current_allowance as u128;
        msg!("The Current Allowance On This Contract is: {}", approved_amount);
        require!(approved_amount > amount, CollateralVaultError::InsufficientAllowance);

        // Update Allowance
        self.account_collateralizable_allowance./*load_mut()?.*/current_allowance = approved_amount.checked_sub(amount).unwrap() as u64;

        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(reservation_id: u64)]
pub struct ReleaseAllCollateral<'info> {
    
    #[account(
        mut,
        constraint = reserving_contract.key() == collateral_reservations.reserving_contract
        @ CollateralVaultError::UnapprovedCollateralizableContract
    )]
    pub reserving_contract: Signer<'info>,

    #[account(
        constraint = token_address.key() == collateral_reservations.token_address
        @ CollateralVaultError::TokenNotSupported
    )]
    pub token_address: InterfaceAccount<'info, Mint>,

    /// CHECK : SAFE as validation is done
    #[account(
        mut,
        constraint = account_address.key() == collateral_reservations.account_address
        @ CollateralVaultError::WrongAccountAddress
    )]
    pub account_address: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"collateral_reservations", reservation_id.to_le_bytes().as_ref()],
        bump
    )]
    pub collateral_reservations: Account<'info, CollateralReservations>,

    #[account(
        mut,
        seeds = [b"account_balance_pda", account_address.key().as_ref(), token_address.key().as_ref()],
        bump
    )]
    pub account_balance_pda: Account<'info, AccountsBalance>,
}

