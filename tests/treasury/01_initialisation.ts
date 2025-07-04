// As this is currently implemented, initialising the token mint in-line, we cannot run it in parallel with the voting_tokens tests.
// One way of getting around this is to run the tests by launching a local test validator
// ```solana-test-validator -r --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s .anchor/metaplex.so```
// and using
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor run test-treasury```
// to run only the tests in the tests/treasury folder (check Anchor.toml for implementation of that instrustion).
// This does no building/deploying on its own, so those instructions must be executed separately:
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor build --no-idl```
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor idl build -p treasury -o target/idl/treasury.json -t target/types/treasury.ts```
// ```RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor deploy```

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from "@solana/web3.js";
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    getAssociatedTokenAddressSync,
} from "@solana/spl-token";

import { TreasuryProgram } from "../../target/types/treasury";
import { VotingTokens } from "../../target/types/voting_tokens";

describe("treasury", async () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    const provider = anchor.getProvider();
    const connection = provider.connection;
    const program = anchor.workspace.Treasury as Program<TreasuryProgram>;

    // Initialise the voting tokens mint
    const mintfn = async () => {
      const mintProgram = anchor.workspace.VotingTokens as Program<VotingTokens>;// Metaplex Constants
      const METADATA_SEED = "metadata";
      const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
  
      const MINT_SEED = "mint";
    
      const signer = Keypair.generate();
      const mintPda = PublicKey.findProgramAddressSync(
        [Buffer.from(MINT_SEED)],
        mintProgram.programId
      );
      const metadataPda = PublicKey.findProgramAddressSync(
          [
              Buffer.from(METADATA_SEED),
              TOKEN_METADATA_PROGRAM_ID.toBuffer(),
              mintPda[0].toBuffer(),
          ],
          TOKEN_METADATA_PROGRAM_ID
        );
    
      const mint_accounts = {
        signer: signer.publicKey,
        mint: mintPda[0],
        metadata: metadataPda[0],
        system_program: SystemProgram.programId,
        token_program: TOKEN_PROGRAM_ID,
        token_metadata_program: TOKEN_METADATA_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      };

      // Airdrop
      let tx_airdrop = new Transaction();
      tx_airdrop.instructions = [
        SystemProgram.transfer({
          fromPubkey: provider.publicKey,
          toPubkey: signer.publicKey,
          lamports: 0.1*LAMPORTS_PER_SOL,
        })
      ];
      await provider.sendAndConfirm(tx_airdrop, []);

      const params_correct = {
        name: "AuthensusVotingToken",
        symbol: "AUTHVOTE",
        uri: "",
        decimals: 9,
      };

      const tx_init_mint = await mintProgram.methods.init(params_correct)
          .accounts({ ...mint_accounts })
          .signers([signer])
          .rpc()
          .then(confirm);
      
      return mintPda[0];
    }
  
    const signer = Keypair.generate();
    const mint = await mintfn();
    const treasuryPda = PublicKey.findProgramAddressSync(
      [Buffer.from("treasury")],
      program.programId
    );
    const voting_token_account = getAssociatedTokenAddressSync(mint, treasuryPda[0], true);
  
    const init_accounts = {
      signer: signer.publicKey,
      treasury: treasuryPda[0],
      voting_token_account,
      mint,
      token_program: TOKEN_PROGRAM_ID,
      associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
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

        //


        // ------ EXECUTE ------

        const tx = await program.methods.initialise()
            .accounts({ ...init_accounts })
            .signers([signer])
            .rpc()
            .then(confirm)
            .then(log);


        // ----- EVALUATE ------

        console.log("Your transaction signature", tx);
    });
});
