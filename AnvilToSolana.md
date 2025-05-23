### PROGRAM STRUCTURE

# COLLATERAL VAULT PROGRAM

1. Initialize a set of Collateral Tokens The COLLATERAL VAULT supports

- Create an a PDA account that will hold these supported collateral tokens. Each supported Tokens mapping in this PDA should be of type: CollateralToken {userCumulativeDeposits, bool isEnabled}. So now, [tokenAddress,CollateralToken] key & value pair mapping is what is supposed to be stored in a SupportedTokenRegistry PDA

2. Initialize a set of Collateralizable programs the COLLATERAL VAULT has approved. So needs to have PDA registry of ApprovedCollateralizableContracts. This is to ensure only approved Collateralizable programs can interact with the COLLATERAL VAULT

# DepositAndApprove instruction

- parameters: an array of token addresses, array of token amounts, and a specified collateralizable contract address.

* First Check: Ensure provided collateralizable contract address is found in the PDA registry of ApprovedCollateralizableContracts.
* Call some `depositToAccount()` public instruction
* loop through each amounts array, and calls some private `authorizedModifyCollateralizableTokenAllowance` function.

# DepositToAccount() instruction

- Parameters: account address, array of tokens, array of amounts

* First Check: Ensure the arrays of tokens and amounts are the same
* Loop through the array of tokens, and for each token, call some internal `_deposit(), parameters: caller_address, caller_address, token, amount` instruction

# ClaimCollateral() instruction

- Parameters: reservationId, amountToReceive, toAddress, a bool \_releaseRemainder

* returns 2 uints by calling an internal function `_claimCollateral()` with same parameters in same order

# DepositFromAccount() instruction
