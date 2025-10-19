import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CollateralVault} from "../target/types/collateral_vault";
import { LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import { createMint, getAccount, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { BN } from "bn.js";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { airdropSol } from "@lightprotocol/stateless.js";
//import { token } from "@coral-xyz/anchor/dist/cjs/utils";

describe("Collateral Vault", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.collateralVault as Program<CollateralVault>;
  //const anvilProgram = anchor.workspace.Anvil 

  const admin = provider.wallet;

  // Let's Set Mints
  let usdcTokenMint: PublicKey;
  let daiTokenMint: PublicKey;
  let ethTokenMint: PublicKey;

  // Let's Set CollateralizableContracts
  const collateralizableContract1 = anchor.web3.Keypair.generate();
  const collateralizableContract2 = anchor.web3.Keypair.generate();

  let callerOfDeposit =  anchor.web3.Keypair.generate();

  // Airdrop SOL
  async function airdropSol(publicKey, solAmount) {
    const airdropSig = await provider.connection.requestAirdrop(
      publicKey,
      solAmount * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);
  }

  before(async () => {
    await airdropSol( callerOfDeposit.publicKey, 5);
    await airdropSol( collateralizableContract1.publicKey, 5);
  })

  it("TEST 1 :::   Initialize Supported Tokens And Collateralizable Contracts Registry", async () => {
    // Add your test here.
    const [tokenRegistryPDA, tokenRegistryBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("supported_token_registry")],
        program.programId
    );

    const [collateralizableContractsPDA, collateralizableContractsBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateralizable_contracts")],
      program.programId
    );

    const [collateralReservationsNoncePDA, ] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_reservations_nonce")],
      program.programId
    );

    await program.methods
      .initTokensAndCollateralizableRegistry()
      .accounts({
        admin: admin.publicKey,
        //@ts-ignore
        tokensRegistry: tokenRegistryPDA,
        collateralizableContracts: collateralizableContractsPDA,
        collateralReservationsNonce: collateralReservationsNoncePDA,
        systemProgram: SystemProgram.programId
      })
      .signers([])
      .rpc();

    console.log("Available accounts:", Object.keys(program.account));

    // Assertions
    const collateralizableContractsData = await program.account.collateralizableContracts.fetch(collateralizableContractsPDA);
    expect(collateralizableContractsData.collaterizableContracts.length).to.eq(0);
    const collateralReservationsNonceData = await program.account.collateralReservationsNonce
      .fetch(collateralReservationsNoncePDA);
    expect(collateralReservationsNonceData.nonce.toNumber()).to.eq(0);
    //const tokenRegistryData = await program.account
    
  });


  it("TEST 2 :::   Add Token To Supported Tokens", async () => {
    // Let's Get A Token Mint
    usdcTokenMint = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      admin.publicKey,
      6,
    );

    daiTokenMint = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      admin.publicKey,
      9,
    );

    // GET PDAs
    const [tokenRegistryPDA, tokenRegistryBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("supported_token_registry")],
        program.programId
    );

    // Let's call the update token registry function
    await program.methods
      .updateSupportedTokens(usdcTokenMint, { add: {}})
      .accounts({
        admin: admin.publicKey,
        //@ts-ignore
        tokensRegistry: tokenRegistryPDA,
      })
      .signers([admin.payer])
      .rpc();

    await program.methods
      .updateSupportedTokens(daiTokenMint, { add: {}})
      .accounts({
        admin: admin.publicKey,
        //@ts-ignore
        tokensRegistry: tokenRegistryPDA,
      })
      .signers([admin.payer])
      .rpc();
  })


  it("TEST 3 :::   Approve Collateralizable Contract", async () => {

    // Get PDAs
    const [collateralizableContractsPDA, collateralizableContractsBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateralizable_contracts")],
      program.programId
    );

    await program.methods
      .updateCollateralizableContracts(collateralizableContract1.publicKey, { add: {}})
      .accounts({
        admin: admin.publicKey,
        //@ts-ignore
        collateralizableContracts: collateralizableContractsPDA,
      })
      .signers([admin.payer])
      .rpc();

    const allCollaterizableContracts = await program.account.collateralizableContracts.fetch(collateralizableContractsPDA);
    expect(allCollaterizableContracts.collaterizableContracts.length).to.eq(1);
  })


  it("TEST 4 :::   Deposit And Approve Function Call Testing", async () => {
    
    // Get PDAs
    const [collateralizableContractsPDA, collateralizableContractsBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateralizable_contracts")],
      program.programId
    );

    const [tokenRegistryPDA, tokenRegistryBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("supported_token_registry")],
        program.programId
    );

    const [accountsBalancePDA, accountsBalanceBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("caller"), callerOfDeposit.publicKey.toBuffer()],
      program.programId
    );

    // Create Associated Token Accounts of usdc and dai tokens to the caller
    const callerUsdcATA = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      admin.payer,
      usdcTokenMint,
      callerOfDeposit.publicKey,
    );
    const callerDaiATA = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      admin.payer,
      daiTokenMint,
      callerOfDeposit.publicKey
    );

    // Let's Mint To Caller ATAs
    await mintTo(
      provider.connection,
      admin.payer,
      usdcTokenMint,
      callerUsdcATA.address,
      admin.publicKey,
      500 * 10 ** 6,
      [admin.payer]
    );

    await mintTo(
      provider.connection,
      admin.payer,
      daiTokenMint,
      callerDaiATA.address,
      admin.publicKey,
      500 * 10 ** 6,
      [admin.payer]
    );

    const [accountBalancePDAUSDc, accountBalancePDABump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("account_balance_pda"),
        callerOfDeposit.publicKey.toBuffer(),
        usdcTokenMint.toBuffer()
      ],
      program.programId
    );
    const [accountBalancePDADai, accountBalancePDADaiBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("account_balance_pda"),
        callerOfDeposit.publicKey.toBuffer(),
        daiTokenMint.toBuffer()
      ],
      program.programId
    );

    const [allowancePDAUsdc, ] = PublicKey.findProgramAddressSync(
      [
        callerOfDeposit.publicKey.toBuffer(),
        collateralizableContract1.publicKey.toBuffer(),
        usdcTokenMint.toBuffer()
      ],
      program.programId
    );
  
    const [allowancePDADai, allowancePDADaiBump] = PublicKey.findProgramAddressSync(
      [
        callerOfDeposit.publicKey.toBuffer(),
        collateralizableContract1.publicKey.toBuffer(),
        daiTokenMint.toBuffer()
      ],
      program.programId
    );

    const bankTokenVault = anchor.web3.Keypair.generate();

    const bankTokenVaultATAUsdc = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      admin.payer,
      usdcTokenMint,
      bankTokenVault.publicKey
    );

    const bankTokenVaultATADai = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      admin.payer,
      daiTokenMint,
      bankTokenVault.publicKey
    );

    const remainingAccounts = [];
    remainingAccounts.push(
      {pubkey: daiTokenMint, isWritable: false, isSigner: false},
      {pubkey: usdcTokenMint, isWritable: false, isSigner: false},
      {pubkey: callerUsdcATA.address, isWritable: true, isSigner: false},
      {pubkey: bankTokenVaultATAUsdc.address, isWritable: true, isSigner: false},
      {pubkey: callerDaiATA.address, isWritable: true, isSigner: false},
      {pubkey: bankTokenVaultATADai.address, isWritable: true, isSigner: false},
      {pubkey: accountBalancePDAUSDc, isWritable: true, isSigner: false},
      {pubkey: accountBalancePDADai, isWritable: true, isSigner: false},
      {pubkey: allowancePDADai, isWritable: true, isSigner: false},
      {pubkey: allowancePDAUsdc, isWritable: true, isSigner: false},
    )

    const bankDaiATA = await getAccount(provider.connection, bankTokenVaultATADai.address);
    const callerDai = await getAccount(provider.connection, callerDaiATA.address);
    const bankUsdcATA = await getAccount(provider.connection, bankTokenVaultATAUsdc.address);
    const callerUsdc = await getAccount(provider.connection, callerUsdcATA.address);
    console.log("The Caller's DAI ATA Balance Before Deposit is: ", Number(callerDai.amount));
    console.log("The Caller's USDC ATA Balance Before Deposit is: ", Number(callerUsdc.amount));
    console.log("The Bank DAI ATA Vault Before Deposit is: ", Number(bankDaiATA.amount));
    console.log("The Bank USDC ATA Vault Balance Before Deposit is: ", Number(bankUsdcATA.amount));

    const allowancePDADaiData = await provider.connection.getAccountInfo(allowancePDADai);
    //console.log("The Allowance PDA Dai Data is: ", allowancePDADaiData);
    // Call instruction
    await program.methods
      .depositAndApprove( [usdcTokenMint, daiTokenMint], [new BN(200 * 10 ** 6), new BN(300 * 10 ** 6)], collateralizableContract1.publicKey)
      .accounts({
        caller: callerOfDeposit.publicKey,
        //@ts-ignore
        tokensRegistry: tokenRegistryPDA,
        collateralizableContracts: collateralizableContractsPDA,
        bankTokenVault: bankTokenVault.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID
      })
      .remainingAccounts(remainingAccounts)
      .signers([callerOfDeposit])
      .rpc();

    // Get Account Data
    const bankDaiATAAfter = await getAccount(provider.connection, bankTokenVaultATADai.address);
    const callerDaiAfter = await getAccount(provider.connection, callerDaiATA.address);
    const bankUsdcATAAfter = await getAccount(provider.connection, bankTokenVaultATAUsdc.address);
    const callerUsdcAfter = await getAccount(provider.connection, callerUsdcATA.address);
    console.log("The Caller's DAI ATA Balance After Deposit is: ", Number(callerDaiAfter.amount));
    console.log("The Bank DAI ata Vault After Deposit is: ", Number(bankDaiATAAfter.amount));
    console.log("The Caller's USDC ATA Balance After Deposit is: ", Number(callerUsdcAfter.amount));
    console.log("The Bank USDC ata Vault After Deposit is: ", Number(bankUsdcATAAfter.amount));

    //const allowancePDADaiDataAfter = await provider.connection.getAccountInfo(allowancePDADai);
    //console.log("The Allowance PDA Dai Data After is: ", allowancePDADaiDataAfter.data.toString());
    //const collateralizableContract1Data = await program.account.collateralizableContracts
      //.fetch(collateralizableContract1.publicKey);
    
    //const collateralizableContract1AllowanceDAIData = await program.account.accountsBalance.fetch(
      //allowancePDADai
    //);
    //const tokenRegistryData = await program.account.

  //const accountBalanceDaiCContract1Data = await program.account.accountsBalance.fetch(accountBalancePDADai);
    const accountsBalanceDaiCContract1Data = await provider.connection.getAccountInfo(collateralizableContract1.publicKey);
      //console.log("The Accounts Balance is: ", accountsBalanceDaiCContract1Data.data);
    //console.log("Collateralizable Contract 1 Data is: ", collateralizableContract1Data);
    //console.log("Collateralizable Contract 1 Allowance Data is: ", collateralizableContract1AllowanceDAIData);
    //console.log("Account Balance DAI for CC 1 is: ", accountBalanceDaiCContract1Data);
    //console.log("Account Collateral Balance Available is: ", accountBalanceDaiCContract1Data.collateralBalance.available.toNumber());
    //console.log("Account Collateral Balance Reserved is : ", accountBalanceDaiCContract1Data.collateralBalance.reserved.toNumber());
    
  })


  it("TEST 5 ::: Deposit To Account Function Call Testing", async () => {})

  it("TEST 6 ::: Reserve Collateral Function Call Testing", async () => {

    // Set PDAs
    const [accountBalancePDAUSDc, ] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("account_balance_pda"),
        callerOfDeposit.publicKey.toBuffer(),
        usdcTokenMint.toBuffer()
      ],
      program.programId
    );
    const [accountBalancePDADai, ] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("account_balance_pda"),
        callerOfDeposit.publicKey.toBuffer(),
        daiTokenMint.toBuffer()
      ],
      program.programId
    );

    const [allowancePDAUsdc, ] = PublicKey.findProgramAddressSync(
      [
        callerOfDeposit.publicKey.toBuffer(),
        collateralizableContract1.publicKey.toBuffer(),
        usdcTokenMint.toBuffer()
      ],
      program.programId
    );
  
    const [allowancePDADai, ] = PublicKey.findProgramAddressSync(
      [
        callerOfDeposit.publicKey.toBuffer(),
        collateralizableContract1.publicKey.toBuffer(),
        daiTokenMint.toBuffer()
      ],
      program.programId
    );

    const [collateralizableContractsPDA, ] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateralizable_contracts")],
      program.programId
    );

    const [tokenRegistryPDA, ] = PublicKey.findProgramAddressSync(
        [Buffer.from("supported_token_registry")],
        program.programId
    );

    const [collateralReservationsNoncePDA, ] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_reservations_nonce")],
      program.programId
    );

    const [collateralReservationsPDA1, ] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_reservations"), new BN(1).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const [collateralReservationsPDA2, ] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_reservations"), new BN(1).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    // Call Instruction
    await program.methods
      .reserveCollateralExt(callerOfDeposit.publicKey, new BN(150 * 10 ** 6))
      .accounts({
        accountAddress: callerOfDeposit.publicKey,
        reservingContract: collateralizableContract1.publicKey,
        tokenAddress: usdcTokenMint,
        //@ts-ignore
        tokensRegistry: tokenRegistryPDA,
        collateralizableContracts: collateralizableContractsPDA,
        accountBalancePda: accountBalancePDAUSDc,
        accountCollateralizableAllowance: allowancePDAUsdc,
        collateralReservationsNonce: collateralReservationsNoncePDA,
        collateralReservations: collateralReservationsPDA1,
        systemProgram: SystemProgram.programId
      })
      .signers([collateralizableContract1])
      .rpc();

    /*await program.methods
      .reserveCollateralExt(callerOfDeposit.publicKey, new BN(150))
      .accounts({
        accountAddress: callerOfDeposit.publicKey,
        reservingContract: collateralizableContract1.publicKey,
        tokenAddress: daiTokenMint,
        //@ts-ignore
        tokensRegistry: tokenRegistryPDA,
        collateralizableContracts: collateralizableContractsPDA,
        accountBalancePda: accountBalancePDADai,
        accountCollateralizableAllowance: allowancePDADai,
        collateralReservationsNonce: collateralReservationsNoncePDA,
        collateralReservations: collateralReservationsPDA2,
        systemProgram: SystemProgram.programId
      })
      .signers([collateralizableContract1])
      .rpc();*/

      const accountBalanceUSDCdata = await program.account.accountsBalance.fetch(accountBalancePDAUSDc);
      const accountBalanceDaidata = await program.account.accountsBalance.fetch(accountBalancePDADai);
      const allowanceDaiInfo = await program.account.accountCollateralizableAllowance.fetch(allowancePDADai);
      const allowanceUsdcInfo = await program.account.accountCollateralizableAllowance.fetch(allowancePDAUsdc);

      console.log("The Available USDC Collateral Balance Is: ", accountBalanceUSDCdata.collateralBalance.available.toNumber());
      console.log("The Reserved USDC Collateral Balance Is: ", accountBalanceUSDCdata.collateralBalance.reserved.toNumber());
      console.log("The Available DAI Collateral Balance Is: ", accountBalanceDaidata.collateralBalance.available.toNumber());
      console.log("The Reserved DAI Collateral Balance Is : ", accountBalanceDaidata.collateralBalance.reserved.toNumber());

      console.log("The DAI Allowance On The Collateralizable Contract 1 is: ", allowanceDaiInfo.currentAllowance.toNumber());
      console.log("The USDC Allowance On The Collateralizable Contract 1 is: ", allowanceUsdcInfo.currentAllowance.toNumber());
      
  })
});