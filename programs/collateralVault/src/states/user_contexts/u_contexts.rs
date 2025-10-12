use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::
    {Mint, TokenInterface}
};

use crate::states::{ errors::*, accounts::*};


#[derive(Accounts)]
pub struct DepositAndApprove<'info> {

    #[account(mut)]
    pub caller: Signer<'info>,

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

    /*  We Need To Create an AccountsBalance PDA for each user
    #[account(
        init_if_needed,
        payer = caller,
        seeds = [b"accounts_balance", caller.key().as_ref()],
        bump,
        space = 8 + AccountsBalance::INIT_SPACE,
    )]
    pub accounts_balance: Account<'info, AccountsBalance>,*/

    // Create Program's Bank Vault To Hold Each Deposited Tokens
    /// CHECKED: This ATA will be created Natively, So Safe
    pub bank_token_vault: AccountInfo<'info>,

    // System Program Needed To Create The AccountsBalance PDA
    pub system_program: Program<'info, System>,

    // Associated Token Program Needed
    pub associated_token_program: Program<'info, AssociatedToken>,

    // Token Program Needed To Make The Transfer
    pub token_program: Interface<'info, TokenInterface>
}

impl <'info> DepositAndApprove<'info> {

    // Verify That The Given token Address Is Allowed By The Only Owner.
    fn verify_token_enabled(&self, token_address: Pubkey) -> Result<()> {

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
}


#[derive(Accounts)]
#[instruction(collateralizable_contract_address: Pubkey)]
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
        bump = account_balance_pda.bump_accounts_balance
    )]
    pub account_balance_pda: Account<'info, AccountsBalance>,

    #[account(
        mut,
        seeds = [account_address.key().as_ref(), collateralizable_contract_address.key().as_ref(), token_address.key().as_ref()],
        bump
    )]
    pub account_collateralizable_allowance: Account<'info, AccountCollateralizableAllowance>,

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

        let approved_amount: u128 = self.account_collateralizable_allowance.current_allowance as u128;
        
        require!(approved_amount > amount, CollateralVaultError::InsufficientAllowance);

        // Update Allowance
        self.account_collateralizable_allowance.current_allowance = approved_amount.checked_sub(amount).unwrap() as u64;

        Ok(())
    }
}




pub fn only_enabled_collateral_tokens<'info>(ctx: &Context/*<'_, '_, '_, '_, */<DepositAndApprove<'info>>, token_addresses: &Vec<Pubkey>) -> Result<()> {

            for i in token_addresses.iter(){
                ctx.accounts.verify_token_enabled(*i)?;
            }
        
            Ok(())
    }