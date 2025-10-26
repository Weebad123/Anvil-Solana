use anchor_lang::prelude::*;
use anchor_spl::{token_interface::{Mint, TokenAccount, transfer_checked, TransferChecked}, 
associated_token::AssociatedToken};

use crate::states::{accounts::*, errors::*};

#[derive(Accounts)]
pub struct Withdraw<'info> {

    #[account(mut)]
    pub withdrawer: Signer<'info>,

    #[account()]
    pub token_address: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::authority = withdrawer.key(),
        associated_token::mint = token_address
    )]
    pub withdrawer_ata: InterfaceAccount<'info, TokenAccount>,

     #[account(
        mut,
        seeds = [b"account_balance_pda", withdrawer.key().as_ref(), token_address.key().as_ref()],
        bump
    )]
    pub account_balance_pda: Account<'info, AccountsBalance>,

    /// CHECKED: This ATA will be created Natively, So Safe
    pub bank_token_vault: AccountInfo<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Withdraw<'info> {

    pub fn initiate_withdraw(&mut self, amount: u128) -> Result<()> {

        Ok(())
    }
}