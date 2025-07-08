import { readFileSync } from "fs";
import { homedir } from "os";
import path from "path";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction, type GetVersionedTransactionConfig } from "@solana/web3.js";
import nacl from "tweetnacl";
import naclUtil from "tweetnacl-util";

import {assert } from "chai";

import { Market } from "../../target/types/market";
import { VotingTokens } from "../../target/types/voting_tokens";

describe("market", () => {
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

    // Facets
    const truthfulness = { truthfulness: {} };
    const originality = { originality: {} };
    const authenticity = { authenticity: {} };

    const [authensusTokenKP, admin, firstBettor] = Array.from({ length: 3 }, () => Keypair.generate());
    // const authensusTokenKP = Keypair.generate();
    const treasury = loadKeypairFromFile("authensus_treasury_keypair.json");
    const marketPda = PublicKey.findProgramAddressSync(
        [
            Buffer.from("market"),
            authensusTokenKP.publicKey.toBuffer(),
        ],
        program.programId
    );
    const escrowPda = PublicKey.findProgramAddressSync(
        [
            Buffer.from("escrow"),
            authensusTokenKP.publicKey.toBuffer(),
            Buffer.from("truthfulness"),
        ],
        program.programId
    );
    const pollPda = PublicKey.findProgramAddressSync(
        [
            Buffer.from("poll"),
            authensusTokenKP.publicKey.toBuffer(),
            Buffer.from("truthfulness"),
        ],
        program.programId
    );
    const initialiserPda = PublicKey.findProgramAddressSync(
        [
            Buffer.from("bettor"),
            authensusTokenKP.publicKey.toBuffer(),
            Buffer.from("truthfulness"),
            firstBettor.publicKey.toBuffer(),
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

    const log_with_cost = async (signature: string): Promise<string> => {
        console.log(
            `Link: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
        );

        let config: GetVersionedTransactionConfig = {
            commitment: "finalized",
            maxSupportedTransactionVersion: 0,
        };
        let result = await connection.getTransaction(signature, config);
        console.log(`Transaction fee: ${result?.meta?.fee}`);
        console.log(`Transaction compute units: ${result?.meta?.computeUnitsConsumed}`);

        return signature;
    };
    

    it("Starts Market", async () => {

        // ------- SETUP -------

        // Airdrop SOL
        let airdropTx = new Transaction();
        airdropTx.instructions = Array.from(
            [admin, firstBettor],
            (s) => SystemProgram.transfer({
                fromPubkey: provider.publicKey,
                toPubkey: s.publicKey,
                lamports: 0.01*LAMPORTS_PER_SOL,
            })
        );
        await provider.sendAndConfirm(airdropTx, []);

        // Initialise market
        const authensusToken = authensusTokenKP.publicKey;
        const facets = [ truthfulness, originality, authenticity ];
        const timeout = 7*24*60*60;    // 7 days
        const init_market_accounts = {
            admin: admin.publicKey,
            market: marketPda[0],
            system_program: SystemProgram.programId,
        };
        await program.methods.initialiseMarket(authensusToken, facets, new anchor.BN(timeout))
            .accounts({ ...init_market_accounts })
            .signers([admin])
            .rpc()
            .then(confirm);

        const marketParams = {
            authensusToken: authensusToken,
            facet: truthfulness,
        };
        const amount = 1_000_000;
        const direction = true;
        
        const startMarketAccounts = {
            signer: firstBettor.publicKey,
            treasury: treasury.publicKey,
            market: marketPda[0],
            escrow: escrowPda[0],
            poll: pollPda[0],
            initialiser: initialiserPda[0],
            system_program: SystemProgram.programId,
        };

        const messageString = authensusToken.toString() + "1" + "truthfulness" + firstBettor.publicKey.toString() + "1000000";
        const encodedMessage: Uint8Array = naclUtil.decodeUTF8(messageString);// getUtf8Encoder().encode(messageString);
        const signedMessage: Uint8Array = nacl.sign.detached(encodedMessage, treasury.secretKey); // await signBytes(treasury.secretKey, encodedMessage);

        // const decodedSignature = getBase58Decoder().decode(signedMessage);
        const verified = nacl.sign.detached.verify(encodedMessage, signedMessage, treasury.publicKey.toBytes()); // await verifySignature(treasury.publicKey, signedMessage, encodedMessage);

        assert(verified);


        // ------ EXECUTE ------

        await program.methods.startMarket(marketParams, new anchor.BN(amount), direction, [...signedMessage])
            .accounts({ ...startMarketAccounts })
            .signers([firstBettor])
            .rpc()
            .then(confirm)
            .then(log_with_cost);


        // ----- EVALUATE ------

        assert(true);

    });

});
