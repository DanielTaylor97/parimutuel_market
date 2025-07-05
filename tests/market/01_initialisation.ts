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

import { readFileSync } from "fs";
import { homedir } from "os";
import path from "path";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";

import { ParimutuelMarket } from "../../target/types/market";
import { VotingTokens } from "../../target/types/voting_tokens";

describe("market", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const connection = provider.connection;
  const program = anchor.workspace.Market as Program<ParimutuelMarket>;

  // Function to read keypair from file
  function loadKeypairFromFile(filePath: string): Keypair {
    const resolvedPath = path.resolve(
      filePath.startsWith("~") ? filePath.replace("~", homedir()) : filePath
    );
    const loadedKeyBytes = Uint8Array.from(
      JSON.parse(readFileSync(resolvedPath, "utf8"))
    );
    return Keypair.fromSecretKey(loadedKeyBytes);
  }

  // Initialise the voting tokens mint
  const mintfn = async () => {
    const mintProgram = anchor.workspace.VotingTokens as Program<VotingTokens>;

    const MINT_SEED = "mint";
    const mintPda = PublicKey.findProgramAddressSync(
      [Buffer.from(MINT_SEED)],
      mintProgram.programId
    );
    
    return [mintPda[0], mintProgram.programId];
  }

  const [authensusTokenKP, admin] = Array.from({ length: 2 }, () => Keypair.generate());
  // const authensusTokenKP = Keypair.generate();
  const treasury = loadKeypairFromFile("authensus_treasury_keypair.json");
  const marketPda = PublicKey.findProgramAddressSync(
    [
      Buffer.from("market"),
      authensusTokenKP.publicKey.toBuffer(),
    ],
    program.programId
  );

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
      `Link: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  };

  // Facets
  const truthfulness = { truthfulness: {} };
  const originality = { originality: {} };
  const authenticity = { authenticity: {} };


  it("Airdrop", async () => {

    let tx = new Transaction();

    tx.instructions = Array.from(
      [admin],
      (s) => SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: s.publicKey,
        lamports: 0.1*LAMPORTS_PER_SOL,
      })
    );

    await provider.sendAndConfirm(tx, []).then(log);

  });


  it("Initialises Marketplace", async () => {

    // ------- SETUP -------

    const [mint, _mintProgramId] = await mintfn();
    const treasuryAta = getAssociatedTokenAddressSync(mint, treasury.publicKey, true);
    const init_marketplace_accounts = {
      treasury: treasury.publicKey,
      treasury_token_account: treasuryAta,
      mint,
      token_program: TOKEN_PROGRAM_ID,
      associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
      system_program: SystemProgram.programId,
    };


    // ------ EXECUTE ------

    const tx = await program.methods.initMarketplace()
      .accounts({ ...init_marketplace_accounts })
      .signers([])
      .rpc()
      .then(confirm)
      .then(log);


    // ----- EVALUATE ------

    console.log("Marketplace init signature", tx);

  });


  it("Initialises Market", async () => {

    // ------- SETUP -------

    const authensusToken = authensusTokenKP.publicKey;
    const facets = [ truthfulness, originality, authenticity ];
    const timeout = 7*24*60*60*1000;

    const init_market_accounts = {
      admin: admin.publicKey,
      market: marketPda[0],
      system_program: SystemProgram.programId,
    };


    // ------ EXECUTE ------

    const tx = await program.methods.initialiseMarket(authensusToken, facets, new anchor.BN(timeout))
      .accounts({ ...init_market_accounts })
      .signers([admin])
      .rpc()
      .then(confirm)
      .then(log);


    // ----- EVALUATE ------

    console.log("Market init signature", tx);

  });

});
