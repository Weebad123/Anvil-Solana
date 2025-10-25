use anchor_lang::prelude::*;
use crate::states::{self, errors::CollateralVaultError};

pub trait CollateralUtils<'info> {

    fn get_tokens_registry(&self) -> Account<'info, states::accounts::TokenRegistry>;

    fn get_collateralizable_contracts(&self) -> Account<'info, states::accounts::CollateralizableContracts>;

    fn get_account_collateralizable_allowance(&self) -> Account<'info, states::accounts::AccountCollateralizableAllowance>;

    fn get_account_balance_pdas(&self) -> (Account<'info, states::accounts::AccountsBalance>, Account<'info, states::accounts::AccountsBalance>);

    fn get_account_address(&self) -> Pubkey;

    fn get_reserving_contract_address(&self) -> Pubkey;

    fn verify_token_enabled(&self, token_address: Pubkey) -> Result<()> {

        let token_registry = self.get_tokens_registry();

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

    fn require_collateralizable_and_decrease_approved_amount(&mut self, amount: u128) -> Result<()> {
        
        // Don't Revert If Same; Function Can Be Used By Pool Contracts for Transfers
        if self.get_account_address() == self.get_reserving_contract_address() {
            return Ok(());
        }

        require!(self.get_collateralizable_contracts().collaterizable_contracts.contains(
            &self.get_reserving_contract_address()),
            CollateralVaultError::UnapprovedCollateralizableContract
        );

        let approved_amount = self.get_account_collateralizable_allowance().current_allowance as u128;
        require!(approved_amount > amount, CollateralVaultError::InsufficientAllowance);

        // Update Allowance
        let _ = self.get_account_collateralizable_allowance().reload();
        self.get_account_collateralizable_allowance().current_allowance = approved_amount
            .checked_sub(amount).unwrap() as u64;
        Ok(())
    }

    fn transfer_collateral(&mut self, amount: u128) -> Result<()> {

        let (mut user_account_balance_pda, mut contract_account_balance_pda) = self.get_account_balance_pdas();
        
        if user_account_balance_pda.key() == contract_account_balance_pda.key() {
            return Ok(());
        }

        // Ensure User Has Enough Collateral
        require!(user_account_balance_pda.collateral_balance.available as u128 > amount,
            CollateralVaultError::InsufficientCollateral);
        
        contract_account_balance_pda.collateral_balance.available = contract_account_balance_pda
            .collateral_balance.available.checked_add(amount as u64).unwrap();

        user_account_balance_pda.collateral_balance.available = user_account_balance_pda
            .collateral_balance.available.checked_sub(amount as u64).unwrap();
        
        Ok(())
    }
}