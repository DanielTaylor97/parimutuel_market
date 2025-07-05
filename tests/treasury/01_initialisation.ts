/*

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
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { assert, expect } from "chai";

import { TreasuryProgram } from "../../target/types/treasury";
import { VotingTokens } from "../../target/types/voting_tokens";

describe("treasury", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    const provider = anchor.getProvider();
    const connection = provider.connection;
    const program = anchor.workspace.Treasury as Program<TreasuryProgram>;

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
  
    const [signer, coparty] = Array.from({ length: 2 }, () => Keypair.generate());
    const treasuryPda = PublicKey.findProgramAddressSync(
      [Buffer.from("treasury")],
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
        `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
      );
      return signature;
    };


    it("Airdrop", async () => {
  
      let airdrop_tx = new Transaction();
  
      airdrop_tx.instructions = Array.from(
        [signer, coparty],
        (kp) => SystemProgram.transfer({
          fromPubkey: provider.publicKey,
          toPubkey: kp.publicKey,
          lamports: 0.1*LAMPORTS_PER_SOL
        })
      );
  
      await provider.sendAndConfirm(airdrop_tx, []).then(log);
  
    });


    it("Initialises", async () => {

      // ------- SETUP -------
    
      const [mint, _mintProgramId] = await mintfn();
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


    it("Transacts", async () => {

      // ------- SETUP -------
    
      const [mint, _mintProgramId] = await mintfn();
      const voting_token_account = getAssociatedTokenAddressSync(mint, treasuryPda[0], true);

      const deposit_amount = 1_000_000;
      const withdrawal_amount = 100_000;
    
      const transaction_accounts = {
        signer: signer.publicKey,
        coparty: coparty.publicKey,
        treasury: treasuryPda[0],
        votingTokenAccount: voting_token_account,
        token_program: TOKEN_PROGRAM_ID,
        system_program: SystemProgram.programId,
        associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
      };


      // ------ EXECUTE ------

      const deposit_tx = await program.methods.deposit(new anchor.BN(deposit_amount))
          .accounts({ ...transaction_accounts })
          .signers([signer, coparty])
          .rpc()
          .then(confirm)
          .then(log);


      // ----- EVALUATE ------

      assert(await connection.getBalance(coparty.publicKey) == 0.1*LAMPORTS_PER_SOL - deposit_amount);
      console.log("Deposit signature", deposit_tx);


      // ------ EXECUTE ------

      console.log(`Signer: ${signer.publicKey}`);
      console.log(`Coparty: ${coparty.publicKey}`);

      const reimburse_tx = await program.methods.reimburse(new anchor.BN(withdrawal_amount))
          .accounts({ ...transaction_accounts })
          .signers([signer, coparty])
          .rpc()
          .then(confirm)
          .then(log);


      // ----- EVALUATE ------

      assert(await connection.getBalance(coparty.publicKey) == 0.1*LAMPORTS_PER_SOL - deposit_amount + withdrawal_amount);
      console.log("Withdrawal signature", reimburse_tx);
    });
});

*/
