use anchor_lang::prelude::*;
//use anchor_spl::token::Token;



// Supported Collateral Token
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, PartialEq)]
pub struct CollateralToken {
    pub user_cumulative_balance: u64,

    pub is_enabled: bool
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, PartialEq)]
pub struct CollateralBalance {
    pub available: u64,

    pub reserved: u64,
}

// We Need To Create an [accountsBalance] PDA to track the total Available and Reserved of A user's token Balance
#[account]
#[derive(InitSpace)]
pub struct AccountsBalance {
    pub collateral_balance: CollateralBalance,

    pub bump_accounts_balance: u8,

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
pub struct TokenRegistry {
    pub token_registry_bump: u8,

    pub collateral_tokens: Vec<(Pubkey, CollateralToken)>,
}

impl TokenRegistry {
    const MAX_TOKENS: usize = 20;
    pub const SPACE: usize = 8 + 1 + 4 + (TokenRegistry::MAX_TOKENS * (32 + 8 + 1));
}


#[account]
#[derive(InitSpace)]
pub struct CollateralizableContracts {
    pub collaterizable_contracts_bump: u8,

    #[max_len(100)]
    pub collaterizable_contracts: Vec<Pubkey>,
}