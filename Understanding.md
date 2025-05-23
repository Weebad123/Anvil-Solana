###### COLLATERAL VAULT

1.  Collateral Vault provides access to authorized `collateralizable contracts` via the `ICollateral.sol` interface.

- To have access to the `COLLATERAL VAULT`, i need to write a [collateralizable_contracts] by implementing the `ICollateral.sol interface`.

* Things `Collateralizable Contracts` Can do on behalf of an account:
  - reserve collateral
  - claim collateral
  - modify collateral
  - pool collateral
  - release collateral
  # Of course, this action is only possible if the [account] and the [Collateral_Vault_Governance] has approved this Collateralizable Contracts

# Withdrawal Fee (0.5%)=== applied to any collateral amount that exits the [COLLATERAL_VAULT]

- Updates to Withdrawal Fee will be unapplicable to collateral in `CollateralReservation` at time of update.
- If existing tokens are diallowed in future governance actions, `existing CollateralReservation ought to be honored, but no new collateral reservations may be created for that token`.

2. # DepositAndApprove(), external

- parameters: array of tokens and their respective amounts, and address of the collateralizable contract (this is to give it some allowance)

* First Check: Ensures collateralizable contract address is approved by Governance
* Calls some `depositToAccount(), parameters: msg.sender, address array, amounts array` public function
* loop through the [amounts] array parameters, and calls a private `authorizedModifyCollateralizableTokenAllowance` to either increase or decrease the specified token allowance of the msg.sender on the collateralizable contract.

3. # DepositToAccount(), external on ICollateralDepositTarget interface

- parameters: account address, array of tokens, array of amounts.

* First Check: Ensures both arrays of tokens and amounts are same
* Loop through each token in the array, and calls some internal `_deposit(): parameters: msg.sender, msg.sender, token, amount` function

4. # ClaimCollateral(), external on ICollateral interface

- parameters: reservationId, amountToReceive, toAddress, a bool \_releaseRemainder

* Returns 2 uints by calling an internal function `_claimCollateral()` with same parameters

5. # DepositFromAccount(), external on ICollateral interface

- Parameters: account address, token address, amount, bytes collateralizableDepositApprovalSignature

* First Check: Requires caller to be an approved Collateralizable contract
* Calls some internal `_verifyDepositApprovalSignature()` with the same parameters above
* Query allocated Allowance (account address has alloted to collateralizable contract(caller in this case))
* If the queried allowance is less than the amount parameter, there is a call made to some private `authorizedModifyCollateralizableTokenAllowance(, , , byAmount)` with byAmount parameter set to [(allowance-amount)].
* Calls some internal `_deposit(): parameters: account_address, account_address, token_address, amount` function

6. # ModifyCollateralizableTokenAllowance(), external

- Parameters: collateralizable contract address, token address, int byAmount

* First check : Reverts if byAmount > 0 && collateralizable contract is unverified. `//!audit if byAmount is <=0, approval check is bypassed`
* Calls some private `_authorizedModifyCollateralizableTokenAllowance(msg.sender, collateralizable, token, byAmount)` function

7. # ModifyCollateralizableTokenAllowanceWithSignature(), external on ICollateral interface

- Parameters: account address, collateralizable contract, token address, and adjustable amount

* First Check : Reverts if adjustableAmount > 0 && collateralizable contract is unverified. `!audit: if adjustableAmount <= 0, approval check is bypassed.`

* Calls some `_modifyCollateralizableTokenAllowanceWithSignature()` private function.

8. # ModifyCollateralReservation(), external on ICollateral interface

- Parameters: reservationId, int byAmount

* Calls some private `_modifyCollateralReservation()` function with same args

9. # PoolCollateral, external on ICollateral interface

- Parameters: account address, token , amount

* First Check: modifier that checks if token is enabled or not
* Calls some internal `requireCollateralizableAndDecreaseApproveAmount()` function to check if collateralizable is approved, and given allowance is valid
* Calls some private `_transferCollateral()` function with the params: token, account, amount, msg.sender

10. # ReleaseAllCollateral, external on ICollateral interface

- Parameters: reservationId

* Calls some internal `_releaseAllCollateral()` function with same parameter

11. # ReserveClaimableCollateral(), external on ICollateral interface

- Parameters: account address, token address, claimable amount

* assign a new totalResevedAmount by calling amountWithFee on the Pricing contract with params: claimable amount, withdrawalFeeBasisPoints
* Calls some private `_reserveCollateral()` function with params: msg.sender, account address, token address, totalReservedAmount, claimable amount

12. # ReserveCollateral(), external on ICollateral interface

- Parameters: account address, token address, amount

* assign a new claimable variable by calling amountBeforeFee on the Pricing contract with params: amount, withdrawalFeeBasisPoints
* Calls some private `_reserveCollateral()` function with params: msg.sender, account address, token address, claimable, amount

13. # TransferCollateral(), external on ICollateral interface

- Parameters: token address, amount, destination address

* Calls an internal `_transferCollateral(, msg.sender, , )` function with same parameters:

14. # UpgradeAccount(), external

- Parameters: targetContractAddress, arrays of token addresses, arrays of amounts

* First Check: Ensures targetContractAddress is an approved address by governance
* Second Check: Arrays length equality check
* Iterate through each tokenAddresses, and if available collateral balance is more than amount to transfer, update the total Cumulative user deposits on collateralToken, reduce the collateral available, and forceApprove the targetContractAddress for the amount
* Calls the `depositToAccount()` external function on the ICollateralDepositTarget interface.

15. # Withdraw(), external on ICollateral interface

- Parameters: address tokenAddress, amount, destination address

* First Check: Non-Zero amount specified to withdraw, specified amount is more than user available collateral, destination address not a zero address
* Decrement available collateral on a user account by withdrawing amount, and collateral cumulative deposits
* Get withdrawal fee on withdrawing amount, and transfer the (amount - fee) to specified destination address via safeTransfer()

[Governance_Functions]

16. # setWithdrawalFeeBasisPoints(), external and onlyOwner

- Parameters: newFeeBasisPoints

* First Check: Ensures New Fee is less than 10%, and update withdrawalFeeBasisPoints to this new value

17. # upsertCollateralizableContractsApproval(), external and onlyOwner

- Parameters: an array of collateralizableContractsApprovalConfig

* iterate throught the array, and for each, gets the collateral address, and then sets the isApproved flag on it if not a zero address
* In a try/catch block, checks for the interface ID of the contract address, and uses some assembly language to check for an offset and updates. catch some random bytes

18. # upsertCollateralTokens(), external and onlyOwner

- Parameters: an array of collateralTokenConfigs

* Calls some private `authorizedUpsertCollateralTokens()` with same parameters

19. # upsertCollateralUpgradeContractAddress(), external and onlyOwner

- Parameters: collateralUpgradeContractAddress(), some bool isApproved.

* Sets the arg address in the permittedCollateralUpgradeContracts to the specified isApproved value

* In a try/catch block, checks for the interface of the contract address, and returns some bool supported if interface matches. if not, revert right away. revert in catch block too which just catches some random bytes.

20. # withdrawFromProtocolBalance() , external and onlyOwner

- Parameters: array of tokenAddresses, array of amounts, and destination address

* First Checks: destination address not a zero address, array lengths match exactly.
* Iterates through the tokenAddresses, and transfers (token balance in contract - cumulativeUserDeposits) to the destination address via safeTransfer. ensures there are enough tokens in contract.

[Internal_Private_Functions]

21. # authorizedModifyCollateralizableTokenAllowance(), private

- Parameters: account address, collateralizable address, token address, and byAmount

* Get current allowance on account, and if increasing allowance, either increase it by specified byAmount, or set to type(uint256).max. And if decreasing allowance, either decrease by specified byAmount or set to 0.

22. # authorizedUpsertCollateralTokens(), private

- Parameters: an array of collateralTokenConfig

* Iterates through the array, and set each collateralToken configurations (the cumulativeUserDeposits, and isEnabled flag).

23. # ClaimCollateral (), internal

- Parameters: reservationId, amountToReceive, toAddress, releaseRemainder? ....... Return values: remainingReservedCollateral, remainingClaimableCollateral

* First Checks: Non-zero claiming amount, not a zero Address destination, only collateralizable contract whose reservationId is provided can be msg.sender
* Gets the claimableTokenAmount on the CollateralReservations, and then ensures it's less than the claimingAmount
* Calculates remaining collateral balance after this claiming amount.
* If user is claiming all the claimable collateral, code sets [releaseRemainder] to true, [remainingReservedCollateral] to 0, and [amountWithFee] = tokenAmount on CollateralReservations
* For partial collateral claiming, calculates remainingReservedCollateral = amountWithFee on Pricing contract with remainingClaimable and feeBasisPoints as args, and
  [amountWithFee] = tokenAmount - remainingReservedCollateral
* updates the cumulativeUserBalance of the claiming token.
* For full claiming, decrease user's collateral balance of reserve with amountWithFee, and increase that of available with 0. For partial claiming, decrease user's collateral balance of reserve with amountWithFee, but doesnot increase that of available. `//!@audit: possible flaw: ascertain impact`
* Calculates fee for event emission, and makes a safeTransfer token transfer

23. # Deposit(), internal

- Parameters: transferSource, account address, token address, amount

* Updates User's [available] CollateralBalance, and token's [cumulativeUserBalance]
* Call safeTransferFrom() to deposit tokens from the transferSource to this contract

24. # ModifyCollateralReservation(), internal

- Parameters: reservationId, byAmount .......... Return Vaules: reserved Collateral and claimable collateral

* First Check: Ensure there is a non-zero token Amount on the CollateralReservations of the specified reservationId.
* Second: returns the tokenAmount and claimableTokenAmount on the CollateralReservations for a 0 byAmount value. Check that msg.sender is authorized via CollateralReservations.collateralizableContractAddress
* If Decreasing Collateral, check the decrement value is within range, and subsequently updates the CollateralReservations.tokenAmount, and user's reserved and available
* If Increasing Collateral, verify token address is enabled, and Calls some [requireCollateralizableAndDecreaseApprovalAmount()] function that checks that collateralizable contract is approved, and has sufficient approval to move tokens from user
* Ensures there is enough user collateral available, and then updates the CollateralReservations.tokenAmount, and user's Collateral Account Balance
* Calculates claimableTokenAmount based on updated reservedCollateral using Pricing.amountBeforeFee(), and updates the CollateralReservations.claimableTokenAmount

25. # ModifyCollateralizableTokenAllowanceWithSignature(),

- Parameters: account address, collateralizable address, token address, adjustableAllowance, a signature

* Gets the hash by calling Openzeppelin's hashTypedDataV4() function from the cryptography library.
* Checks that the hash and signature is valid using the isValidSignature() function from the Signature library, and revert if invalid
* Calls the authorizedModifyCollateralizableTokenAllowance() function

26. # ReleaseAllCollateral(), internal

- Parameters: reservationId

* First Check: Ensures the caller is the collateralizableContract on the CollateralReservations whose ID is provided.
* Increase CollateralBalance.available of user account by the totalCollateral (CollateralReservations.tokenAmount), and decrease the CollateralBalance.reserved by the same CollateralReservations.tokenAmount, and then reset the CollateralReservations struct array.

27. # RequireCollateralizableAndDecreaseApproveAmount(), private

- Parameters: collateralizable contract, account address, token address, adjustable amount

* First Checks: early return if collateralizable and account addresses are same, and checks for approval on collateralizable contract.
* Second Checks: ensures there is more allowance than the decrementing amount
* Update the allowance to this new value (oldAllowance - adjustable amount)

28. # reserveCollateral(), internal

- Parameters: reservingContract, account address, reserveCollateral, claimableCollateral, token address ........ returns reservationId

* First Checks: token Address is an enabled token, and claimableCollateral is non-zero
* Calls [requireCollateralizableAndDecreaseApprovalAmount()] private function with args: reserving contract, account address, token address, and reserveCollateral
* Queries user's Collateral Balance and ensures the CollateralBalance.available is greater than the amount to be reserved(reserveCollateral), and also that the reserveCollateral > claimableCollateral.
* Decrease user CollateralBalance.available, and increase his CollateralBalance.reserved. And increase the global reservation nonce counter to assign ID to the reservations
* It then updates this CollateralReservations for this user, by updating the fields

29. # transferCollateral(), private

- Parameters: fromAddress, token address, destinationAddress, amount

* First Checks: returns early if fromAddress = destinationAddress, and if amount is 0
* Queries fromAddress available CollateralBalance and ensures that it's more than the transferring amount
* Updates the destination available CollateralBalance and the fromAddress available via increasing and decreasing the availables by this amount.

30. # verifyTokensEnabled(), view

- Parameters: collateralToken address

* Reverts if address is not in the CollateralTokens mapping array.

31. # verifyDepositApprovalSignature(), internal virtual

- Parameters: account address, token address, amount, signature

* Gets the hash via Openzeppelin's hashTypedDataV4() function from the crytographic library
* Checks for valid signature by calling the isValidSignatureNow() function on the Signature contract.

#####







#####








#####
DepositAndApprove()     =========>>>>>>>   
1. 
  * Any User Can Call This Function And Supply The Collateralizable Address     OR    
    + User duplicates the arrays, either tokenAddresses or tokenAmounts
      + tokenAddresses[0] = AMP, tokenAddresses[1] = AMP, tokenAddresses[2] = AMP    ===>>>> ???? Nothing will happen
  
  
  * The Collateralizable Address Can Call Itself, And Supply Its Collateralizable Address

`We have 2 types of collateral: Reserve And Claimable collateral`

`{ CollateralBalance { available: is increased via deposits}}....`
` { CollateralToken {cumulativeUserBalance: is increased via deposit}}.......`

- # NOTE:::::: `Whenever we are modifying the reserve collateral, these steps should be taken:`
  `1. If Decreasing  =====>>>CollateralBalance.reserved must decrease by the modifying amount; CollateralBalance.available must increase by modifying amount`. + `CollateralReservations.tokenAmount must decrease by modifying amount`
  `2. If increasing, CollateralBalance.reserved must increase by modifying amount;  CollateralBalance.available must decrease by modifying amount`. + `CollateralReservations.tokenAmount must increase by modifying amount`.




#### Potential Flaw
* If we are moving collateral, decrease the reserved Collateral, and increase the available   ====>>>> so user can get via Withdraw (all available).... In this case, 
cumulative user deposits is decremented.
* OR we can just decrease the reserved collateral, and transfer straight away to user using safeTransfer(tokens), and then updates cumulative User deposits because 
tokens have left the contract.