import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    getAssociatedTokenAddressSync,
} from "@solana/spl-token";

import { TreasuryProgram } from "../../target/types/treasury";

describe("treasury", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    const provider = anchor.getProvider();
    const connection = provider.connection;
    const program = anchor.workspace.Treasury as Program<TreasuryProgram>;
  
    const [signer, mint] = Array.from({ length: 2 }, () => Keypair.generate());
    const treasury = PublicKey.findProgramAddressSync(
      [Buffer.from("treasury")],
      program.programId
    )[0];
    const voting_token_account = getAssociatedTokenAddressSync(mint.publicKey, treasury, true);
  
    const init_accounts = {
      signer: signer.publicKey,
      treasury,
      voting_token_account,
      mint: mint.publicKey,
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
