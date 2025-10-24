use anchor_lang::prelude::*;

use anchor_spl::token_interface::Mint;

use crate::{states::{ errors::*, accounts::*}, utils::CollateralUtils};

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

impl<'info> CollateralUtils<'info> for ReserveCollateral<'info> {

    fn get_tokens_registry(&self) -> Account<'info, TokenRegistry> {
        self.tokens_registry.clone()
    }
    
    fn get_collateralizable_contracts(&self) -> Account<'info, crate::states::accounts::CollateralizableContracts> {
        self.collateralizable_contracts.clone()
    }

    fn get_account_collateralizable_allowance(&self) -> Account<'info, crate::states::accounts::AccountCollateralizableAllowance> {
        self.account_collateralizable_allowance.clone()
    }

    fn get_account_address(&self) -> Pubkey {
        self.account_address.key()
    }

    fn get_reserving_contract_address(&self) -> Pubkey {
        self.reserving_contract.key()
    }

    fn get_account_balance_pdas(&self) -> (Account<'info, crate::states::accounts::AccountsBalance>, Account<'info, crate::states::accounts::AccountsBalance>) {
        (self.account_balance_pda.clone(), self.account_balance_pda.clone())
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


#[derive(Accounts)]
pub struct PoolCollateral<'info> {

    /// CHECK: SAFE TO USE
    #[account(
        mut,
    )]
    pub reserving_contract: AccountInfo<'info>,

    /// CHECK: SAFE TO USE
    #[account(
        mut,
    )]
    pub account_address: AccountInfo<'info>,

    /// CHECK: SAFE TO USE
    #[account()]
    pub token_address: AccountInfo<'info>,

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
    pub account_balance_pda_user: Account<'info, AccountsBalance>,

    #[account(
        mut,
        seeds = [b"account_balance_pda", reserving_contract.key().as_ref(), token_address.key().as_ref()],
        bump
    )]
    pub account_balance_pda_contract: Account<'info, AccountsBalance>,

    #[account(
        mut,
        seeds = [account_address.key().as_ref(), reserving_contract.key().as_ref(), token_address.key().as_ref()],
        bump
    )]
    pub account_collateralizable_allowance: Account<'info, AccountCollateralizableAllowance>,

}

impl<'info> CollateralUtils<'info> for PoolCollateral<'info> {

    fn get_tokens_registry(&self) -> Account<'info, crate::states::accounts::TokenRegistry> {
        self.tokens_registry.clone()
    }

    fn get_collateralizable_contracts(&self) -> Account<'info, crate::states::accounts::CollateralizableContracts> {
        self.collateralizable_contracts.clone()
    }

    fn get_account_collateralizable_allowance(&self) -> Account<'info, crate::states::accounts::AccountCollateralizableAllowance> {
        self.account_collateralizable_allowance.clone()
    }

    fn get_account_balance_pdas(&self) -> (Account<'info, crate::states::accounts::AccountsBalance>, Account<'info, crate::states::accounts::AccountsBalance>) {
        (self.account_balance_pda_user.clone(), self.account_balance_pda_contract.clone())
    }

    fn get_account_address(&self) -> Pubkey {
        self.account_address.key()
    }

    fn get_reserving_contract_address(&self) -> Pubkey {
        self.reserving_contract.key()
    }

}

