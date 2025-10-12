import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CollateralVault} from "../target/types/collateral_vault";
import { LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
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

    await program.methods
      .initTokensAndCollateralizableRegistry()
      .accounts({
        admin: admin.publicKey,
        //@ts-ignore
        tokensRegistry: tokenRegistryPDA,
        collateralizableContracts: collateralizableContractsPDA,
        systemProgram: SystemProgram.programId
      })
      .signers([])
      .rpc();

    console.log("Available accounts:", Object.keys(program.account));

    // Assertions
    const collateralizableContractsData = await program.account.collateralizableContracts.fetch(collateralizableContractsPDA);
    expect(collateralizableContractsData.collaterizableContracts.length).to.eq(0);
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
      {pubkey: callerDaiATA.address, isWritable: true, isSigner: false},
      {pubkey: bankTokenVaultATADai.address, isWritable: true, isSigner: false},
      {pubkey: accountBalancePDAUSDc, isWritable: true, isSigner: false},
      {pubkey: accountBalancePDADai, isWritable: true, isSigner: false},
      {pubkey: allowancePDADai, isWritable: true, isSigner: false},
      {pubkey: allowancePDAUsdc, isWritable: true, isSigner: false},
    )

    // Call instruction
    await program.methods
      .depositAndApprove( [/*usdcTokenMint,*/ daiTokenMint], [/*new BN(200 * 10 ** 6),*/ new BN(300 * 10 ** 6)], collateralizableContract1.publicKey)
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
  })
});