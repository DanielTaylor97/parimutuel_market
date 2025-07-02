import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ParimutuelMarket } from "../target/types/market";

describe("market", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Market as Program<ParimutuelMarket>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialiseMarket().rpc();
    console.log("Your transaction signature", tx);
  });
});
