use anchor_lang:: prelude::*;
use bytemuck::{Zeroable, Pod};

pub const MAX_TOKENS: usize = 20;


// Supported Collateral Token
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, PartialEq)]
pub struct CollateralToken {
    pub user_cumulative_balance: u64,

    pub is_enabled: bool
}

//#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, /*Zeroable, Pod,*/ InitSpace, Default, PartialEq)]
pub struct CollateralBalance {
    pub available: u64,

    pub reserved: u64,
}

// We Need To Create an [accountsBalance] PDA to track the total Available and Reserved of A user's token Balance
#[account/*(zero_copy)*/]
//#[repr(C)]
#[derive(InitSpace, Default)]
pub struct AccountsBalance {
    pub collateral_balance: CollateralBalance,

    pub bump_accounts_balance: u8,

    pub padding: [u8;7],

}


// Create An [AccountCollateralizableTokenAllowance] PDA to track a user's token allowance for a particular Collateralizable Contract
#[account/*(zero_copy)*/]
//#[repr(C)]
#[derive(InitSpace, Default)]
pub struct AccountCollateralizableAllowance {
    pub current_allowance: u64,

    pub bump_account_collateralizable: u8,

    pub padding: [u8;7],
}

// Create A Program's Bank Vault Authority To Oversee The Contract's Token Balance
#[account]
#[derive(InitSpace)]
pub struct BankVaultAuthority {
    #[max_len(10)]
    pub trusted_authorities: Vec<Pubkey>,

    pub bank_vault_authority_bump: u8,
}

#[account]
#[derive( InitSpace, Copy, PartialEq)]
pub struct TokenEntry {
    pub token_mint: Pubkey,

    pub collateral_token: CollateralToken
}
/* 
#[account]
#[derive(InitSpace)]
pub struct TokenRegistry {
    pub token_registry_bump: u8,

    #[max_len(20)]
    pub collateral_tokens: Vec<TokenEntry>,
}
*/

#[account]
#[derive(InitSpace)]
pub struct CollateralReservationsNonce {
    pub nonce: u64,
}

#[account]
pub struct TokenRegistry {
    pub token_registry_bump: u8,

    pub collateral_tokens: Vec<(Pubkey, CollateralToken)>,
}

impl anchor_lang::Space for TokenRegistry {
    
    const INIT_SPACE: usize = 8 + 1 + 4 + (20 * (32 + 8 + 1));
}


#[account]
#[derive(InitSpace)]
pub struct CollateralizableContracts {
    pub collaterizable_contracts_bump: u8,

    #[max_len(100)]
    pub collaterizable_contracts: Vec<Pubkey>,
}


#[account]
#[derive(InitSpace)]
pub struct CollateralReservations {
    pub reserving_contract: Pubkey,

    pub account_address: Pubkey,

    pub token_address: Pubkey,

    pub withdrawal_fee: u16,

    pub reserved_collateral: u128,

    pub claimable_collateral: u128,
}