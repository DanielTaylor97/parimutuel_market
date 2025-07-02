import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";

import { ParimutuelMarket } from "../target/types/market";

describe("market", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const connection = provider.connection;
  const program = anchor.workspace.Market as Program<ParimutuelMarket>;

  const [authensusTokenKP, admin] = Array.from({ length: 4 }, () => Keypair.generate());
  const market = PublicKey.findProgramAddressSync(
    [Buffer.from("market")],
    program.programId
  )[0];

  const init_accounts = {
    admin: admin.publicKey,
    market,
    system_program: SystemProgram.programId,
  };

  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  const log = async (signature: string): Promise<string> => {
    console.log(
      `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  };

  // Facets
  const truthfulness = { truthfulness: {} };
  const originality = { originality: {} };
  const authenticity = { authenticity: {} };

  it("Initialises", async () => {

    // ------- SETUP -------

    const authensusToken = authensusTokenKP.publicKey;
    const facets = [ truthfulness, originality, authenticity ];
    const timeout = 7*24*60*60*1000;


    // ------ EXECUTE ------

    const tx = await program.methods.initialiseMarket(authensusToken, facets, new anchor.BN(timeout))
      .accounts({ ...init_accounts })
      .signers([admin])
      .rpc()
      .then(confirm)
      .then(log);


    // ----- EVALUATE ------

    console.log("Your transaction signature", tx);
  });
});
