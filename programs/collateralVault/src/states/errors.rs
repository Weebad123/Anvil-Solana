use anchor_lang::prelude::*;


#[error_code]
pub enum CollateralVaultError {
    #[msg("Token Already Added As Supported Tokens")]
    AlreadySupportedToken,

    #[msg("Token Is Not Allowed As Collateral:")]
    TokenNotAllowed,

    #[msg("Token Is Not Supported")]
    TokenNotSupported,

    #[msg("Collateralizable Contract Not Found")]
    CollateralizableContractNotFound,

    #[msg("Collateralizable Contract Is Not Approved By Protocol")]
    UnapprovedCollateralizableContract,

    #[msg("Token Addresses And Token Amounts Mismatch")]
    MismatchedTokenAddressesAndAmountsLength,

    #[msg("Token Deposits Overflowed")]
    TokenOverflowError,

    #[msg("Account Balance PDA Not Found")]
    PDAAccountNotFound,

    #[msg("Specified Remaining Token Mints Mismatched With Actual Token")]
    InvalidRemainingMints,

    #[msg("Created Program's Token Vault Mismatches With Remaining Token Vault ATA")]
    MismatchedTokenVaults,

    #[msg("Caller's Derived Token Account Via Remaining Accounts Differs From Token Account")]
    MismatchedTokenAccounts,

    #[msg("Account-Collateralizable PDA Address Mismatches")]
    MismatchedAllowancePDA,

    #[msg("Claimable Collateral Amount Must Not Be Zero")]
    ClaimableAmountZero,

    #[msg("Reserving Contract And Account Address Are Same")]
    SameAsReservingContract,

    #[msg("Allowance on Reserving Contract Is Insufficient")]
    InsufficientAllowance,

    #[msg("Insufficient Collateral On Account Address")]
    InsufficientCollateral,

    #[msg("Token Mint Address Not Found In RemainingAccounts")]
    TokenMintNotFound,
}