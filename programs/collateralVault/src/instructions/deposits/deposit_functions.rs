use anchor_lang::{prelude::*, solana_program::{self, program_pack::Pack, rent::Rent}};
use solana_program::pubkey::Pubkey;

use anchor_spl::{associated_token::{Create, create_idempotent, get_associated_token_address}, token::spl_token::state::Mint, token_interface::{ TransferChecked, transfer_checked}};

use crate::{states::{errors::*, user_contexts::*}, AccountCollateralizableAllowance, AccountsBalance};


// Deposit And Approve
pub fn deposit_plus_approve<'info>(mut ctx: Context<'_, '_, 'info, 'info, DepositAndApprove<'info>>, token_addresses: &Vec<Pubkey>, 
    token_amounts: Vec<u64>, collateralizable_contract_address_to_approve: Pubkey) -> Result<()> {

        // Get The Various Accounts
    let all_collateralizable_contracts = &ctx.accounts.collateralizable_contracts;

    // Ensure The Collateralizable Contract Is Approved
    require!(all_collateralizable_contracts.collaterizable_contracts
        .contains(&collateralizable_contract_address_to_approve), CollateralVaultError::UnapprovedCollateralizableContract);

    // Call Deposit To Account
    let msg_sender = ctx.accounts.caller.clone();
    deposit_to_account(&mut ctx, msg_sender.key(), &token_addresses, &token_amounts)?;

    // Loop Through The Amounts, And Call `AuthorizedModifyCollateralizableTokenAllowance` for each amount
    for i in 0..token_amounts.len() {
        // Call The AuthorizedModifyCollateralizableTokenAllowance
        authorized_modify_collateralizable_token_allowance(
            &mut ctx,
            msg_sender.key(),
            collateralizable_contract_address_to_approve,
            token_addresses[i],
            token_amounts[i] as i64//should be negative: original call => Pricing.safeCastToInt256(amounts[i]). would have to implement the same method.
        )?;
    }
    Ok(())
}


// Deposit To Account
pub fn deposit_to_account<'info>(ctx: &mut Context<'_, '_, 'info, 'info, DepositAndApprove<'info>>, account_address: Pubkey, token_addresses: &Vec<Pubkey>, 
    token_amounts: &Vec<u64>) -> Result<()> {

        let msg_sender = ctx.accounts.caller.clone();
        // Ensure The Arrays Match
        require!(token_addresses.len() == token_amounts.len(), CollateralVaultError::MismatchedTokenAddressesAndAmountsLength);

        // Iterate Through The Token addresses, and For Each Token, Call the deposit function
        for i in 0..token_addresses.len() {
            deposit(
                ctx,
                msg_sender.key(),
                account_address,
                token_addresses[i],
                token_amounts[i]
            )?;
        }
        Ok(())
    }


// Internal Deposit Implementation: This is what has the onlyEnabledCollateralTokens modifier
fn deposit<'info>(ctx: &mut Context<'_, '_, 'info, 'info, DepositAndApprove<'info>>, transfer_source: Pubkey, account_address: Pubkey, token_address: Pubkey, amount: u64) -> Result<()> {
    // let's pull up the CollateralToken associated with the token_addresses
    let collateral_token_storage = &mut ctx.accounts.tokens_registry;

    match collateral_token_storage.collateral_tokens
        .iter_mut()
        .find(|(token, _)| *token == token_address) {
            // Update Cumulative User Balance Of The Collateral Token
            Some((_, collateral_token_info)) => {
                collateral_token_info.user_cumulative_balance = collateral_token_info
                    .user_cumulative_balance.checked_add(amount).ok_or(CollateralVaultError::TokenOverflowError)?;
            },
            None => {}
        }

    // Let's Create The AccountsBalance PDA to track the available and reserved
    let mut account_balance_storage = create_or_get_account_balance_pda(ctx,account_address, token_address)?;

    // Update The Available
    account_balance_storage.collateral_balance.available = account_balance_storage.collateral_balance.available
        .checked_add(amount)
        .ok_or(CollateralVaultError::TokenOverflowError)?;
    
    // Create Program's Token Vault To Hold Deposited Tokens
    let token_mint_info = &ctx.remaining_accounts[0];
    let callers_token_ata_account = &ctx.remaining_accounts[1];
    let bank_token_vault_ata = &ctx.remaining_accounts[2];

    let token_decimals = Mint::unpack(&token_mint_info.try_borrow_data()?)?.decimals;

    require!(token_mint_info.key == &token_address, CollateralVaultError::InvalidRemainingMints);
    let programs_token_vault_ata = get_associated_token_address(
        &ctx.accounts.bank_token_vault.key(),
        &token_address
    );
    let required_accounts_for_vault = Create {
        payer: ctx.accounts.caller.to_account_info(),
        associated_token: bank_token_vault_ata.clone(),
        authority: ctx.accounts.bank_token_vault.to_account_info(),
        mint: token_mint_info.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    };
    let token_cpi = ctx.accounts.associated_token_program.to_account_info();
    let token_ctx = CpiContext::new(token_cpi, required_accounts_for_vault);
    create_idempotent(token_ctx)?;
    require!(bank_token_vault_ata.key == &programs_token_vault_ata, CollateralVaultError::MismatchedTokenVaults);

    // Now, Make Transfer From Caller's ATA To Programs Token Vault
    let callers_token_ata = get_associated_token_address(
        &transfer_source,
        &token_address
    );
    require!(callers_token_ata_account.key == &callers_token_ata, CollateralVaultError::MismatchedTokenAccounts);
    let transfer_accounts = TransferChecked {
        from: callers_token_ata_account.clone(),
        to: bank_token_vault_ata.clone(),
        mint: token_mint_info.clone(),
        authority: ctx.accounts.caller.to_account_info(),
    };
    let transfer_cpi = CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_accounts);
    transfer_checked(transfer_cpi, amount, token_decimals)?;

    Ok(())
}

// Private approveModifyCollateralizableTokenAllowance
fn authorized_modify_collateralizable_token_allowance<'info>(
    ctx: &mut Context<'_, '_, 'info, 'info, DepositAndApprove<'info>>,
    account_address: Pubkey,
    collateralizable_contract_address: Pubkey,
    token_address: Pubkey,
    by_amount: i64
) -> Result<()> {
    
    let mut new_allowance: u64;

    // GET ACCOUNT_COLLATERALIZABLE_TOKEN_ALLOWANCES
    let current_allowance = 
        account_collateralizable_token_allowance(ctx, account_address, collateralizable_contract_address, token_address)?.current_allowance;

    if by_amount > 0 {
        new_allowance = current_allowance.wrapping_add(by_amount as u64);

        if new_allowance < current_allowance {
            // This means there was overflow, but the intention was to increase allowance, so we set allowance to highest integer type
            new_allowance = u64::MAX;
        }
    } else {
        new_allowance = current_allowance.wrapping_sub(by_amount.wrapping_abs() as u64);

        if new_allowance > current_allowance {
            // This means there was underflow, but the intentin was to decrease allowance, so we set the allowance to zero
            new_allowance = 0;
        }
    }

    // Update The Allowance
    if new_allowance != current_allowance {
        account_collateralizable_token_allowance(ctx, account_address, collateralizable_contract_address, token_address)?.current_allowance = new_allowance;
    }

    // EMIT THE EVENT
    Ok(())
}


// Private Instruction To Create The AccountsBalance PDA;
fn create_or_get_account_balance_pda<'info>(ctx: &mut Context<'_, '_, 'info,'info, DepositAndApprove<'info>>, user: Pubkey, token_address: Pubkey) -> 
    Result<Account<'info, AccountsBalance>> {

    // Let's Try And Get The PDA
    let owner_address = user.key();
    let token = token_address.key();

    let (account_balance_pda, account_pda_bump) = Pubkey::find_program_address(
        &[b"account_balance_pda", owner_address.as_ref(), token.as_ref()],
        ctx.program_id
    );

    let pda_account_balance = ctx.remaining_accounts
        .iter()
        .find(|account| account.key() == account_balance_pda)
        .ok_or(CollateralVaultError::PDAAccountNotFound)?;


    if pda_account_balance.data_is_empty() || pda_account_balance.lamports() == 0 {

        let rent = Rent::get()?;
        let space = 8 + AccountsBalance::INIT_SPACE;
        let lamports = rent.minimum_balance(space);

        let account_creation_ix = solana_program::system_instruction::create_account(
            ctx.accounts.caller.key,
            &account_balance_pda,
            lamports,
            space as u64,
           // &solana_program::system_program::ID
           &*ctx.program_id
        );

        let accounts_needed_for_creation = &[
            ctx.accounts.caller.to_account_info(),
            pda_account_balance.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ];

        let seeds = &[
            b"account_balance_pda",
            owner_address.as_ref(),
            token.as_ref(),
            &[account_pda_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        solana_program::program::invoke_signed(
            &account_creation_ix,
            accounts_needed_for_creation,
            signer_seeds
        )?;

        
    }

    let disc = AccountsBalance::DISCRIMINATOR;
    let initial_state = AccountsBalance {
        ..Default::default()
    };
    let serialized = initial_state.try_to_vec()?;
    {
        let mut data = pda_account_balance.try_borrow_mut_data()?;
        //require!(data.len() > 8, CollateralVaultError::PDAAccountNotFound);

        data[0..8].copy_from_slice(&disc);
        data[8..8 + serialized.len()].copy_from_slice(&serialized);
    }

    msg!("The owner of this PDA is: {}", pda_account_balance.to_account_info().owner);
    msg!("The data length of this PDA is: {}", pda_account_balance.to_account_info().data_len());
    
    // Deserialize The Account, and Return The Deserialized Account
    let account_balance_storage: Account<AccountsBalance> = Account::try_from(pda_account_balance)?;

        
    Ok(account_balance_storage)
}


fn account_collateralizable_token_allowance<'info>(ctx: &mut Context<'_, '_, 'info,'info, DepositAndApprove<'info>>, 
    account_address: Pubkey, collateralizable_contract_address: Pubkey, token_address: Pubkey) -> Result<Account<'info, AccountCollateralizableAllowance>> {

        let (account_collateralizable_token_allowance_pda, account_collateralizable_token_bump) = Pubkey::find_program_address(
             &[account_address.as_ref(), collateralizable_contract_address.as_ref(), token_address.as_ref()],
            ctx.program_id,
        );
        let pda_account_collateralizable_allowance = ctx.remaining_accounts
            .iter()
            .find(|account| account.key() == account_collateralizable_token_allowance_pda)
            .ok_or(CollateralVaultError::MismatchedAllowancePDA)?;

        require!(pda_account_collateralizable_allowance.key() == account_collateralizable_token_allowance_pda, CollateralVaultError::MismatchedAllowancePDA);

        let rent = Rent::get()?;
        let space = 8 + AccountCollateralizableAllowance::INIT_SPACE;
        let lamports = rent.minimum_balance(space);

        
        if pda_account_collateralizable_allowance.data_is_empty() || pda_account_collateralizable_allowance.lamports() == 0 {

            let account_creation_ix = solana_program::system_instruction::create_account(
                ctx.accounts.caller.key,
                &account_collateralizable_token_allowance_pda,
                lamports,
                space as u64,
                //&solana_program::system_program::ID
                &*ctx.program_id
            );

            let accounts_needed_for_creation = &[
                ctx.accounts.caller.to_account_info(),
                pda_account_collateralizable_allowance.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ];

            let seeds = &[
                account_address.as_ref(), 
                collateralizable_contract_address.as_ref(), 
                token_address.as_ref(),
                &[account_collateralizable_token_bump]
            ];

            let signer_seeds = &[&seeds[..]];

            solana_program::program::invoke_signed(
                &account_creation_ix,
                accounts_needed_for_creation,
                signer_seeds
            )?;
        }

        let disc = AccountCollateralizableAllowance::DISCRIMINATOR;
        let initial_state = AccountCollateralizableAllowance {
            ..Default::default()
        };
        let serialized = initial_state.try_to_vec()?;
        {
            let mut data = pda_account_collateralizable_allowance.try_borrow_mut_data()?;
            data[0..8].copy_from_slice(&disc);
            data[8..8 + serialized.len()].copy_from_slice(&serialized);
        }

        let account_collateralizable_token_allowance_storage: Account<AccountCollateralizableAllowance> = Account::try_from(
            &pda_account_collateralizable_allowance
        )?;
        

        Ok(account_collateralizable_token_allowance_storage)
    }
