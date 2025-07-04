// As this is currently implemented, initialising the token mint in-line, we cannot run it in parallel with the voting_tokens tests.
// One way of getting around this is to run the tests by launching a local test validator
// ```solana-test-validator -r --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s .anchor/metaplex.so```
// and using
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor run test-market```
// to run only the tests in the tests/market folder (check Anchor.toml for implementation of that instrustion).
// This does no building/deploying on its own, so those instructions must be executed separately:
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor build --no-idl```
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor idl build -p market -o target/idl/market.json -t target/types/market.ts```
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor deploy```

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";

import { ParimutuelMarket } from "../../target/types/market";

describe("market", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const connection = provider.connection;
  const program = anchor.workspace.Market as Program<ParimutuelMarket>;

  const [authensusTokenKP, admin] = Array.from({ length: 2 }, () => Keypair.generate());
  const market = PublicKey.findProgramAddressSync(
    [
      Buffer.from("market"),
      authensusTokenKP.publicKey.toBuffer(),
    ],
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


  it("Airdrop", async () => {

    let tx = new Transaction();

    tx.instructions = [
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: admin.publicKey,
        lamports: 0.1*LAMPORTS_PER_SOL,
      })
    ];

    await provider.sendAndConfirm(tx, []).then(log);

  });


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
