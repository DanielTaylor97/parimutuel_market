// Due to conflicts with the proc-macro2 crate regarding versions of solana, rustup and anchor, we cannot just use ```anchor build``` and ```anchor test```.
// To build we must first use
// ```anchor build --no-idl```
// followed by building each of the programs individually using nightly build from before the versioning issue:
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor idl build -p voting_tokens -o target/idl/voting_tokens.json -t target/types/voting_tokens.ts```
// or
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor idl build -p market -o target/idl/market.json -t target/types/market.ts```
//
//
//
// As this is currently implemented we cannot run it in parallel with the market tests as they create their own mint for testing, which will cause reinitialisation errors.
// Run each test suite separately using
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor test --run tests/voting_tokens/*.ts```
// or
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor test --run tests/market/*.ts```

import { readFileSync } from "fs";
import { homedir } from "os";
import path from "path";
import * as anchor from "@coral-xyz/anchor";
import { AnchorError, Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {assert, expect } from "chai";

import { Market } from "../../target/types/market";
import { VotingTokens } from "../../target/types/voting_tokens";

describe("Initialise Market", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const connection = provider.connection;
  const program = anchor.workspace.Market as Program<Market>;

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

  // Initialise the voting tokens mint
  const mintfn = async () => {
    const mintProgram = anchor.workspace.VotingTokens as Program<VotingTokens>;
  
    const initParams = {
      name: "AuthensusVotingToken",
      symbol: "AUTHVOTE",
      uri: "",
      decimals: 9,
    };

    const MINT_SEED = "mint";
    const mintPda = PublicKey.findProgramAddressSync(
      [Buffer.from(MINT_SEED)],
      mintProgram.programId
    );
    const METADATA_SEED = "metadata";
    const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
    const metadataPda = PublicKey.findProgramAddressSync(
      [
          Buffer.from(METADATA_SEED),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          mintPda[0].toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    const initAccounts = {
      signer: treasury.publicKey,
      mint: mintPda[0],
      metadata: metadataPda[0],
      system_program: SystemProgram.programId,
      token_program: TOKEN_PROGRAM_ID,
      token_metadata_program: TOKEN_METADATA_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    };

    await mintProgram.methods.init(initParams)
        .accounts({ ...initAccounts })
        .signers([treasury])
        .rpc()
        .then(confirm)
        .then(log);
    
    return [mintPda[0], mintProgram.programId];
  }

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
      [admin, treasury],
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
    const init_marketplace_accounts_incorrect = {
      treasury: admin.publicKey,
      treasury_token_account: treasuryAta,
      mint,
      token_program: TOKEN_PROGRAM_ID,
      associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
      system_program: SystemProgram.programId,
    };
    const init_marketplace_accounts = {
      treasury: treasury.publicKey,
      treasury_token_account: treasuryAta,
      mint,
      token_program: TOKEN_PROGRAM_ID,
      associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
      system_program: SystemProgram.programId,
    };


    // ------ EXECUTE ------

    // Should throw the expected error when the wrong signer is given
    try {
      await program.methods.initMarketplace()
        .accounts({ ...init_marketplace_accounts_incorrect })
        .signers([admin])
        .rpc()
        .then(confirm)
        .then(log);
  
      assert(false, "Allowed incorrect signer")
    } catch (err) {
      expect(err).to.be.instanceOf(AnchorError);
      expect((err as AnchorError).error.errorCode.number).to.equal(6201);
      expect((err as AnchorError).error.errorCode.code).to.equal("WrongTreasury");
    }

    const tx = await program.methods.initMarketplace()
      .accounts({ ...init_marketplace_accounts })
      .signers([treasury])
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
    const facets_empty = [ ];
    const timeout = 7*24*60*60;        // 7 days
    const timeout_long = 15*24*60*60;  // 15 days
    const timeout_short = 23*60*60;    // 23 hours

    const init_market_accounts = {
      admin: admin.publicKey,
      market: marketPda[0],
      system_program: SystemProgram.programId,
    };


    // ------ EXECUTE ------

    try {
      await program.methods.initialiseMarket(authensusToken, facets_empty, new anchor.BN(timeout))
        .accounts({ ...init_market_accounts })
        .signers([admin])
        .rpc()
        .then(confirm)
        .then(log);
  
      assert(false, "Allowed empty facets vec")
    } catch(err) {
      expect(err).to.be.instanceOf(AnchorError);
      expect((err as AnchorError).error.errorCode.number).to.equal(6000);
      expect((err as AnchorError).error.errorCode.code).to.equal("NoFacetsProvided");
    }

    try {
      await program.methods.initialiseMarket(authensusToken, facets, new anchor.BN(timeout_long))
        .accounts({ ...init_market_accounts })
        .signers([admin])
        .rpc()
        .then(confirm)
        .then(log);
  
      assert(false, "Allowed too large a timeout")
    } catch(err) {
      expect(err).to.be.instanceOf(AnchorError);
      expect((err as AnchorError).error.errorCode.number).to.equal(6001);
      expect((err as AnchorError).error.errorCode.code).to.equal("TimeoutTooLarge");
    }

    try {
      await program.methods.initialiseMarket(authensusToken, facets, new anchor.BN(timeout_short))
        .accounts({ ...init_market_accounts })
        .signers([admin])
        .rpc()
        .then(confirm)
        .then(log);
  
      assert(false, "Allowed too short a timeout")
    } catch(err) {
      expect(err).to.be.instanceOf(AnchorError);
      expect((err as AnchorError).error.errorCode.number).to.equal(6002);
      expect((err as AnchorError).error.errorCode.code).to.equal("TimeoutTooSmall");
    }

    const tx = await program.methods.initialiseMarket(authensusToken, facets, new anchor.BN(timeout))
      .accounts({ ...init_market_accounts })
      .signers([admin])
      .rpc()
      .then(confirm)
      .then(log);


    // ----- EVALUATE ------

    const onChainMarket = await program.account.market.fetch(marketPda[0]);
    
    assert(onChainMarket.state.initialised, `Market state: ${onChainMarket.state}`);
    assert(onChainMarket.round == 0, `Market on round ${onChainMarket.round}`);
    assert(onChainMarket.timeout.toNumber() == timeout, `Timeout on chain: ${onChainMarket.timeout.toNumber()}; timeout supplied: ${timeout}`);

    console.log("Market init signature", tx);

  });

});
