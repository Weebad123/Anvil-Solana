use anchor_lang::prelude::*;

pub mod instructions;
pub mod states;
pub mod utils;

pub use instructions::*;
pub use states::*;
pub use utils::*;

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


     // Deposit To Account Function
    #[access_control(only_enabled_collateral_tokens(&ctx, &token_addresses))]
    pub fn deposit_to_account_ext<'info>(ctx: Context<'_, '_, 'info, 'info, DepositAndApprove<'info>>, account_address: Pubkey,
    token_addresses: Vec<Pubkey>, token_amounts: Vec<u64>) -> Result<()> {


        // Let's Call The Deposit To Account
        let mut ctx = ctx;
        instructions::deposits::deposit_to_account(&mut ctx, account_address, &token_addresses, &token_amounts)?;

        Ok(())
    }


    // RESERVE COLLATERAL
    //#[access_control(only_enabled_collateral_tokens(&ctx, [&ctx.accounts.token_address.key()]))]
    pub fn reserve_collateral_ext<'info>(ctx: Context<'_, '_, 'info, 'info, ReserveCollateral<'info>>, 
    account_address: Pubkey, amount: u64) -> Result<()> {

        let mut ctx = ctx;
        instructions::reserve_collateral(&mut ctx, account_address, amount)?;

        Ok(())
    }


    // RELEASE ALL COLLATERAL 
    pub fn release_all_collateral_ext(ctx: Context<ReleaseAllCollateral>, reservation_id: u64) -> Result<u128> {

        let total_collateral_released = instructions::release_all_collateral(ctx, reservation_id)?;
        
        Ok(total_collateral_released)
    }


    // RESERVE CLAIMABLE COLLATERAL
    pub fn reserve_claimable_collateral<'info>(ctx: Context<'_, '_, 'info, 'info, ReserveCollateral<'info>>, account_address: Pubkey,
        claimable_amount: u128) -> Result<(u64, u64)> {

            let mut ctx = ctx;
            let (total_amount_reserved, reservation_id) = instructions::reserve_claimable_collateral(&mut ctx, account_address, claimable_amount)?;

            Ok((total_amount_reserved, reservation_id))
        }

    pub fn pool_collateral(ctx: Context<PoolCollateral>, amount: u128) -> Result<()> {

        instructions::pool_collateral(ctx, amount)?;

        Ok(())
    }
}






