# Anvil-Solana



### ERRORS
`Error: Simulation failed. 
Message: Transaction simulation failed: Error processing Instruction 0: instruction changed the balance of a read-only account. 
Logs: 
[
  "Program FCeHw6gbVhHR2NFwxjz2Ayu3VNC8spsJsdq9M5ybQQjV invoke [1]",
  "Program log: Instruction: ReleaseAllCollateralExt",
  "Program FCeHw6gbVhHR2NFwxjz2Ayu3VNC8spsJsdq9M5ybQQjV consumed 7311 of 200000 compute units",
  "Program return: FCeHw6gbVhHR2NFwxjz2Ayu3VNC8spsJsdq9M5ybQQjV gKgSAQAAAAAAAAAAAAAAAA==",
  "Program FCeHw6gbVhHR2NFwxjz2Ayu3VNC8spsJsdq9M5ybQQjV failed: instruction changed the balance of a read-only account"
]. 
Catch the `SendTransactionError` and call `getLogs()` on it for full details.`