use anchor_lang::prelude::*;
//use anchor_spl::token;

use crate::{states::admin_contexts::*, CollateralToken, CollateralVaultError};


pub fn init_tokens_registry_and_collateralizable_contracts(ctx: Context<SupportedTokenRegistryAndCollateralizbleContracts>) -> Result<()> {

    let tokens_registry = &mut ctx.accounts.tokens_registry;
    let collateralizable_contracts = &mut ctx.accounts.collateralizable_contracts;

    // Let's initialize the collection vector
    tokens_registry.collateral_tokens = Vec::new();
    tokens_registry.token_registry_bump = ctx.bumps.tokens_registry;

    // Let's
    collateralizable_contracts.collaterizable_contracts = Vec::new();
    collateralizable_contracts.collaterizable_contracts_bump = ctx.bumps.collateralizable_contracts;
    
    Ok(())
}

pub fn add_or_remove_supported_tokens(ctx: Context<AddOrRemoveSupportedTokens>, token_address: Pubkey, op_type: OpType) -> Result<()> {
    let tokens_registry = &mut ctx.accounts.tokens_registry;

    match op_type {
        OpType::Add => {

            let tokens_exist = tokens_registry.collateral_tokens
                .iter()
                .find(|(address, _)| address == &token_address);
            if tokens_exist.is_none() {
                    // Only  Add When Token Address Is Not In Supported Tokens
                    tokens_registry.collateral_tokens.push((token_address, CollateralToken {
                        user_cumulative_balance: 0,
                        is_enabled: true
                    }))
                }
        },
        OpType::Remove => {

            let token_index =  tokens_registry.collateral_tokens
                .iter()
                .position(|(address, _)| address == &token_address);
                    // Pop that token address from the registry
            if let Some(index) = token_index {
                tokens_registry.collateral_tokens.remove(index);
            }
                    
                
        }
    }
    Ok(())
}



pub fn collateralizable_contract_add(ctx: Context<AddOrRemoveCollateralizableContract>, collateralizable_contract_address: Pubkey, op_type: OpType) -> Result<()> {

    let collateralizable_contracts = &mut ctx.accounts.collateralizable_contracts;

    match op_type {
        OpType::Add => {
            if !collateralizable_contracts.collaterizable_contracts.contains(&collateralizable_contract_address) {
                collateralizable_contracts.collaterizable_contracts.push(collateralizable_contract_address);
            }
        },
        OpType::Remove => {
            if collateralizable_contracts.collaterizable_contracts.contains(&collateralizable_contract_address) {
                let index = collateralizable_contracts
                    .collaterizable_contracts
                    .iter()
                    .position(|s| s == &collateralizable_contract_address)
                    .ok_or(CollateralVaultError::CollateralizableContractNotFound)?;

                // Swap last index with this index
                let last = collateralizable_contracts.collaterizable_contracts.len() - 1;
                if index != last {
                    collateralizable_contracts.collaterizable_contracts[index] = collateralizable_contracts.collaterizable_contracts[last];
                }
                collateralizable_contracts.collaterizable_contracts.pop();
            }
        }
    }
    Ok(())
}


#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub enum OpType {
    Add,

    Remove,
}