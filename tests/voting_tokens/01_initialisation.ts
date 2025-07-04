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

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

import { VotingTokens } from "../../target/types/voting_tokens";

describe("voting_tokens", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.VotingTokens as Program<VotingTokens>;
    const provider = anchor.getProvider();
    const connection = provider.connection;

    // Metaplex Constants
    const METADATA_SEED = "metadata";
    const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

    const MINT_SEED = "mint";
  
    const signer = Keypair.generate();
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
        `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
      );
      return signature;
    };


    it("Airdrop", async () => {
  
      let tx = new Transaction();
  
      tx.instructions = [
        SystemProgram.transfer({
          fromPubkey: provider.publicKey,
          toPubkey: signer.publicKey,
          lamports: 0.1*LAMPORTS_PER_SOL,
        })
      ];
  
      await provider.sendAndConfirm(tx, []).then(log);
  
    });


    it("Initialises", async () => {


        // ------- SETUP -------

        const params_correct = {
            name: "AuthensusVotingToken",
            symbol: "AUTHVOTE",
            uri: "",
            decimals: 9,
        };


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

});
