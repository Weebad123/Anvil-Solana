use anchor_lang::prelude::*;

use crate::states::accounts::*;

// Initialize Token Registry
#[derive(Accounts)]
pub struct SupportedTokenRegistryAndCollateralizbleContracts<'info> {

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + TokenRegistry::SPACE,
        seeds = [b"supported_token_registry"],
        bump,
    )]
    pub tokens_registry: Account<'info, TokenRegistry>,

    #[account(
        init,
        payer = admin,
        space = 8 + CollateralizableContracts::INIT_SPACE,
        seeds = [b"collateralizable_contracts"],
        bump,
    )]
    pub collateralizable_contracts: Account<'info, CollateralizableContracts>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct AddOrRemoveSupportedTokens<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"supported_token_registry"],
        bump = tokens_registry.token_registry_bump
    )]
    pub tokens_registry: Account<'info, TokenRegistry>,
}

#[derive(Accounts)]
pub struct AddOrRemoveCollateralizableContract<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"collateralizable_contracts"],
        bump = collateralizable_contracts.collaterizable_contracts_bump,
    )]
    pub collateralizable_contracts: Account<'info, CollateralizableContracts>,
}