use anchor_lang::prelude::*;

pub mod instructions;
pub mod states;

pub use instructions::*;
pub use states::*;

declare_id!("FCeHw6gbVhHR2NFwxjz2Ayu3VNC8spsJsdq9M5ybQQjV");

#[program]
pub mod collateral_vault {
    use super::*;

   pub fn init_tokens_and_collateralizable_registry(ctx: Context<SupportedTokenRegistryAndCollateralizbleContracts>) -> Result<()> {

    instructions::init_tokens_registry_and_collateralizable_contracts(ctx)?;
    Ok(())
   }

   pub fn update_supported_tokens(ctx: Context<AddOrRemoveSupportedTokens>, token_address: Pubkey, operation_type: OpType) -> Result<()> {

    instructions::add_or_remove_supported_tokens(ctx, token_address, operation_type)?;
    Ok(())
   }

   pub fn update_collateralizable_contracts(ctx: Context<AddOrRemoveCollateralizableContract>, collateralizable_contract_address: Pubkey, operation_type: OpType) -> Result<()> {

    instructions::collateralizable_contract_add(ctx, collateralizable_contract_address, operation_type)?;
    Ok(())
   }


   // Deposit And Approve Function
   #[access_control(only_enabled_collateral_tokens(&ctx, &token_addresses))]
   pub fn deposit_and_approve<'info>(ctx: Context<'_, '_, 'info, 'info, DepositAndApprove<'info>>, token_addresses: Vec<Pubkey>, 
        token_amounts: Vec<u64>, collateralizable_contract_address_to_approve: Pubkey) -> Result<()> {

            // Let's Call The Deposit And Approve
            instructions::deposits::deposit_plus_approve(ctx, &token_addresses, token_amounts, collateralizable_contract_address_to_approve)?;

            Ok(())
        }
}






