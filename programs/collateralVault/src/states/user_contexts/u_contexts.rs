use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::
    TokenInterface
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





pub fn only_enabled_collateral_tokens<'info>(ctx: &Context/*<'_, '_, '_, '_, */<DepositAndApprove<'info>>, token_addresses: &Vec<Pubkey>) -> Result<()> {

            for i in token_addresses.iter(){
                ctx.accounts.verify_token_enabled(*i)?;
            }
        
            Ok(())
    }