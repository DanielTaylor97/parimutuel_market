import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";

import { VotingTokens } from "../../target/types/voting_tokens";

describe("voting_tokens mint", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.VotingTokens as Program<VotingTokens>;
    const provider = anchor.getProvider();
    const connection = provider.connection;

    const MINT_SEED = "mint";

    const recipient = Keypair.generate();
    const mintPda = PublicKey.findProgramAddressSync(
        [Buffer.from(MINT_SEED)],
        program.programId
    );
    const recipientAta = getAssociatedTokenAddressSync(mintPda[0], recipient.publicKey, true);
  
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


    it("Mints", async () => {


        // ------- SETUP -------
  
        // Airdrop
        let tx_airdrop = new Transaction();
        tx_airdrop.instructions = [
            SystemProgram.transfer({
                fromPubkey: provider.publicKey,
                toPubkey: recipient.publicKey,
                lamports: 0.1*LAMPORTS_PER_SOL,
            }),
        ];
        await provider.sendAndConfirm(tx_airdrop, []);

        const amount = 1_000_000_000;

        const accounts = {
            payer: recipient.publicKey,
            mint: mintPda[0],
            recipient: recipientAta,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SystemProgram.programId,
            token_program: TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
        };


        // ------ EXECUTE ------

        const tx = await program.methods.mintTokens(new anchor.BN(amount))
            .accounts({ ...accounts })
            .signers([recipient])
            .rpc()
            .then(confirm)
            .then(log);


        // ----- EVALUATE ------

        const token_balance = (await connection.getTokenAccountBalance(recipientAta)).value.uiAmount;
        expect(token_balance == amount);

        console.log("Your transaction signature", tx);

    });

});
