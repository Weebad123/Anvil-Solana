import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CollateralVault} from "../target/types/collateral_vault";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { token } from "@coral-xyz/anchor/dist/cjs/utils";

describe("Collateral Vault", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.collateral_vault as Program<CollateralVault>;

  const admin = provider.wallet;

  it("TEST 1 :::   Initialize Supported Tokens And Collateralizable Contracts Registry", async () => {
    // Add your test here.
    const tokenRegistryPDA = PublicKey.findProgramAddressSync(
        [Buffer.from("supported_token_registry")],
        program.programId
    );

    await program.methods
      .initTokensAndCollateralizableRegistry()
      .accounts({
        admin: admin.publicKey,
        //@ts-ignore
        tokensRegistry: tokenRegistryPDA,
        systemProgram: SystemProgram.programId
      })
      .signers([])
      .rpc();

    // Assertions
    //const tokensRegistryData = await program.
    
  });


  it("TEST 2 :::   Add Token To Supported Tokens", async () => {})


  it("TEST 3 :::   Approve Collateralizable Contract", async () => {})
});