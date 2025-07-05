// As this is currently implemented we cannot run it in parallel with the other tests as they require their own initialisations of
// the token mint. One way of getting around this is to run the tests by launching a local test validator
// ```solana-test-validator -r --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s .anchor/metaplex.so```
// and using
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor run test-voting-tokens```
// to run only the tests in the tests/voting_tokens folder (check Anchor.toml for implementation of that instrustion).
// This does no building/deploying on its own, so those instructions must be executed separately:
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor build --no-idl```
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor idl build -p voting_tokens -o target/idl/voting_tokens.json -t target/types/voting_tokens.ts```
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor deploy```


import { readFileSync } from "fs";
import { homedir } from "os";
import path from "path";
import * as anchor from "@coral-xyz/anchor";
import { AnchorError, Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert, expect } from "chai";

import { VotingTokens } from "../../target/types/voting_tokens";

describe("voting_tokens init", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.VotingTokens as Program<VotingTokens>;
    const provider = anchor.getProvider();
    const connection = provider.connection;

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

    // Metaplex Constants
    const METADATA_SEED = "metadata";
    const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

    const MINT_SEED = "mint"; 
  
    const signer_random = Keypair.generate();
    const signer = loadKeypairFromFile("authensus_treasury_keypair.json");
    const mintPda = PublicKey.findProgramAddressSync(
      [Buffer.from(MINT_SEED)],
      program.programId
    );
    const metadataPda = PublicKey.findProgramAddressSync(
      [
          Buffer.from(METADATA_SEED),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          mintPda[0].toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    const init_accounts_wrong_signer = {
      signer: signer_random.publicKey,
      mint: mintPda[0],
      metadata: metadataPda[0],
      system_program: SystemProgram.programId,
      token_program: TOKEN_PROGRAM_ID,
      token_metadata_program: TOKEN_METADATA_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    };
    const init_accounts = {
      signer: signer.publicKey,
      mint: mintPda[0],
      metadata: metadataPda[0],
      system_program: SystemProgram.programId,
      token_program: TOKEN_PROGRAM_ID,
      token_metadata_program: TOKEN_METADATA_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
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
        `Link: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
      );
      return signature;
    };
    
    const params_correct = {
      name: "AuthensusVotingToken",
      symbol: "AUTHVOTE",
      uri: "",
      decimals: 9,
    };


    it("Airdrop", async () => {
  
      let tx = new Transaction();
  
      tx.instructions = Array.from(
        [signer, signer_random],
        (s) => SystemProgram.transfer({
          fromPubkey: provider.publicKey,
          toPubkey: s.publicKey,
          lamports: 0.1*LAMPORTS_PER_SOL,
        })
      );
  
      await provider.sendAndConfirm(tx, []).then(log);
  
    });


    it("Catches wrong inputs", async () => {

      // ------- SETUP -------

      const params_incorrect_name = {
        name: "AuthensusToken",
        symbol: "AUTHVOTE",
        uri: "",
        decimals: 9,
      };
      const params_incorrect_symbol = {
          name: "AuthensusVotingToken",
          symbol: "VOTE",
          uri: "",
          decimals: 9,
      };
      const params_incorrect_uri = {
          name: "AuthensusVotingToken",
          symbol: "AUTHVOTE",
          uri: "https://authensus.xyz",
          decimals: 9,
      };
      const params_incorrect_decimals = {
          name: "AuthensusVotingToken",
          symbol: "AUTHVOTE",
          uri: "",
          decimals: 10,
      };


      // ------ EXECUTE/EVALUATE ------

      try {
        await program.methods.init(params_incorrect_name)
          .accounts({ ...init_accounts })
          .signers([signer])
          .rpc()
          .then(confirm)
          .then(log);
  
          assert(false, "should've failed but didn't ")
      } catch (err) {
        expect(err).to.be.instanceOf(AnchorError);
        expect((err as AnchorError).error.errorCode.number).to.equal(6000);
        expect((err as AnchorError).error.errorCode.code).to.equal("WrongName");
      }

      try {
        await program.methods.init(params_incorrect_symbol)
          .accounts({ ...init_accounts })
          .signers([signer])
          .rpc()
          .then(confirm)
          .then(log);
  
          assert(false, "should've failed but didn't ")
      } catch (err) {
        expect(err).to.be.instanceOf(AnchorError);
        expect((err as AnchorError).error.errorCode.number).to.equal(6001);
        expect((err as AnchorError).error.errorCode.code).to.equal("WrongSymbol");
      }

      try {
        await program.methods.init(params_incorrect_uri)
          .accounts({ ...init_accounts })
          .signers([signer])
          .rpc()
          .then(confirm)
          .then(log);
  
          assert(false, "should've failed but didn't ")
      } catch (err) {
        expect(err).to.be.instanceOf(AnchorError);
        expect((err as AnchorError).error.errorCode.number).to.equal(6002);
        expect((err as AnchorError).error.errorCode.code).to.equal("WrongUri");
      }

      try {
        await program.methods.init(params_incorrect_decimals)
          .accounts({ ...init_accounts })
          .signers([signer])
          .rpc()
          .then(confirm)
          .then(log);
  
          assert(false, "should've failed but didn't ")
      } catch (err) {
        expect(err).to.be.instanceOf(AnchorError);
        expect((err as AnchorError).error.errorCode.number).to.equal(6003);
        expect((err as AnchorError).error.errorCode.code).to.equal("WrongDecimals");
      }

    });


    it("Doesn't initialise without the treasury address", async () => {

      try {
        await program.methods.init(params_correct)
          .accounts({ ...init_accounts_wrong_signer })
          .signers([signer_random])
          .rpc()
          .then(confirm)
          .then(log);
      } catch(err) {
        expect(err).to.be.instanceOf(AnchorError);
        expect((err as AnchorError).error.errorCode.number).to.equal(6004);
        expect((err as AnchorError).error.errorCode.code).to.equal("WrongSigner");
      }

    });


    it("Initialises", async () => {

      // ------- SETUP -------
      
      //


      // ------ EXECUTE ------

      const tx = await program.methods.init(params_correct)
          .accounts({ ...init_accounts })
          .signers([signer])
          .rpc()
          .then(confirm)
          .then(log);


      // ----- EVALUATE ------

      console.log("Your transaction signature", tx);

    });

});
