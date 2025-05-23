import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Anvil } from "../target/types/anvil";

describe("anvil", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.anvil as Program<Anvil>;

  it("Is initialized!", async () => {
    
  });
});
