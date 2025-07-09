import { readFileSync } from "fs";
import { homedir } from "os";
import path from "path";
import { startAnchor } from "solana-bankrun";
import { BankrunProvider } from "anchor-bankrun";
import * as anchor from "@coral-xyz/anchor";
import { AnchorError, Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction, type GetVersionedTransactionConfig } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import nacl from "tweetnacl";
import naclUtil from "tweetnacl-util";

import {assert, expect } from "chai";

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

    const [authensusTokenKP, initAdmin, firstBettor, bettor2, bettor3, bettor4, bettor5, bettor6] = Array.from({ length: 8 }, () => Keypair.generate());
    const treasury = loadKeypairFromFile("authensus_treasury_keypair.json");
    const marketPda = PublicKey.findProgramAddressSync(
        [
            Buffer.from("market"),
            authensusTokenKP.publicKey.toBuffer(),
        ],
        program.programId
    );
    // PDAs for escrow and poll
    const [escrowPda, pollPda] = Array.from(
        ["escrow", "poll"],
        (s) => PublicKey.findProgramAddressSync(
            [
                Buffer.from(s),
                authensusTokenKP.publicKey.toBuffer(),
                Buffer.from("truthfulness"),
            ],
            program.programId
        )
    );

    // All the bettors' PDAs for wagers
    const [initialiserPda, bettor2Pda, bettor3Pda, bettor4Pda, bettor5Pda, bettor6Pda] = Array.from(
        [firstBettor, bettor2, bettor3, bettor4, bettor5, bettor6],
        (b) => PublicKey.findProgramAddressSync(
            [
                Buffer.from("bettor"),
                authensusTokenKP.publicKey.toBuffer(),
                Buffer.from("truthfulness"),
                b.publicKey.toBuffer(),
            ],
            program.programId
        )
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
    

    it("Places multiple bets", async () => {

        // ------- SETUP -------

        // Airdrop SOL
        let airdropTx = new Transaction();
        airdropTx.instructions = Array.from(
            [initAdmin, firstBettor, bettor2, bettor3, bettor4, bettor5, bettor6],
            (s) => SystemProgram.transfer({
                fromPubkey: provider.publicKey,
                toPubkey: s.publicKey,
                lamports: 1*LAMPORTS_PER_SOL,
            })
        );
        await provider.sendAndConfirm(airdropTx, []);

        // Initialise market
        const authensusToken = authensusTokenKP.publicKey;
        const facets = [ truthfulness, originality, authenticity ];
        const timeout = 7*24*60*60;    // 7 days
        const init_market_accounts = {
            admin: initAdmin.publicKey,
            market: marketPda[0],
            system_program: SystemProgram.programId,
        };
        await program.methods.initialiseMarket(authensusToken, facets, new anchor.BN(timeout))
            .accounts({ ...init_market_accounts })
            .signers([initAdmin])
            .rpc()
            .then(confirm);

        const marketParams = {
            authensusToken: authensusToken,
            facet: truthfulness,
        };
        const [amount1, amount2, amount3, amount4, amount5, amount6] = Array.from(
            { length: 6 },
            () => {

                return Math.round(Math.random()*100_000)*1_000; 
            }
        );
        const [direction1, direction2, direction3, direction4, direction5, direction6] = Array.from(
            { length: 6 },
            () => {
                return Math.random() > 0.2 // 80% betting on true on avg
            }
        );
        
        // All the account contexts for placing bets
        const startMarketAccounts = {
            signer: firstBettor.publicKey,
            treasury: treasury.publicKey,
            market: marketPda[0],
            escrow: escrowPda[0],
            poll: pollPda[0],
            initialiser: initialiserPda[0],
            system_program: SystemProgram.programId,
        };
        const [bettor2Accounts, bettor3Accounts, bettor4Accounts, bettor5Accounts, bettor6Accounts] = Array.from(
            [
                {a: bettor2, pda: bettor2Pda},
                {a: bettor3, pda: bettor3Pda},
                {a: bettor4, pda: bettor4Pda},
                {a: bettor5, pda: bettor5Pda},
                {a: bettor6, pda: bettor6Pda},
            ],
            (obj) => {
                return {
                    signer: obj.a.publicKey,
                    market: marketPda[0],
                    escrow: escrowPda[0],
                    bettor: obj.pda[0],
                    treasury: treasury.publicKey,
                    system_program: SystemProgram.programId,
                }
            }
        );

        const [signedMessage1, signedMessage2, signedMessage3, signedMessage4, signedMessage5, signedMessage6] = Array.from(
            [
                {account: firstBettor, amount: amount1},
                {account: firstBettor, amount: amount1},
                {account: firstBettor, amount: amount1},
                {account: firstBettor, amount: amount1},
                {account: firstBettor, amount: amount1},
                {account: firstBettor, amount: amount1},
            ],
            (b) => {
                const str = authensusToken.toString() + "1" + "truthfulness" + b.account.publicKey.toString() + b.amount.toString();
                const enc = naclUtil.decodeUTF8(str);
                return nacl.sign.detached(enc, treasury.secretKey)
            }
        );

        // Start the market
        await program.methods.startMarket(marketParams, new anchor.BN(amount1), direction1, [...signedMessage1])
            .accounts({ ...startMarketAccounts })
            .signers([firstBettor])
            .rpc()
            .then(confirm);


        // ------ EXECUTE ------
        
        const tx2 = await program.methods.wager(marketParams, new anchor.BN(amount2), direction2, [...signedMessage2])
            .accounts({ ...bettor2Accounts})
            .signers([bettor2])
            .rpc()
            .then(confirm)
            .then(log);
        
        const tx3 = await program.methods.wager(marketParams, new anchor.BN(amount3), direction3, [...signedMessage3])
            .accounts({ ...bettor3Accounts})
            .signers([bettor3])
            .rpc()
            .then(confirm)
            .then(log);

        // Should throw the expected error when someone tries to make an underdog bet with existing bets in place
        try {
            await program.methods.underdogBet(marketParams, new anchor.BN(amount2), [...signedMessage2])
                .accounts({ ...bettor2Accounts})
                .signers([bettor2])
                .rpc()
                .then(confirm);
        
            assert(false, "Allowed underdog bet after normal wager")
        } catch (err) {
            expect(err).to.be.instanceOf(AnchorError);
            expect((err as AnchorError).error.errorCode.number).to.equal(6605);
            expect((err as AnchorError).error.errorCode.code).to.equal("UnderdogWithOtherBet");
        }
        
        const tx4 = await program.methods.wager(marketParams, new anchor.BN(amount4), direction4, [...signedMessage4])
            .accounts({ ...bettor4Accounts})
            .signers([bettor4])
            .rpc()
            .then(confirm)
            .then(log);
        
        const tx5 = await program.methods.underdogBet(marketParams, new anchor.BN(amount5), [...signedMessage5])
            .accounts({ ...bettor5Accounts})
            .signers([bettor5])
            .rpc()
            .then(confirm)
            .then(log);
        
        const tx6 = await program.methods.wager(marketParams, new anchor.BN(amount6), direction6, [...signedMessage6])
            .accounts({ ...bettor6Accounts})
            .signers([bettor6])
            .rpc()
            .then(confirm)
            .then(log);

        // Should throw the expected error when someone tries to make a normal bet with existing underdog bet
        try {
            await program.methods.wager(marketParams, new anchor.BN(amount5), direction5, [...signedMessage5])
                .accounts({ ...bettor5Accounts})
                .signers([bettor5])
                .rpc()
                .then(confirm);
        
            assert(false, "Allowed normal wager after underdog bet")
        } catch (err) {
            expect(err).to.be.instanceOf(AnchorError);
            expect((err as AnchorError).error.errorCode.number).to.equal(6606);
            expect((err as AnchorError).error.errorCode.code).to.equal("BetWithUnderdogBet");
        }

        const onChainEscrow = await program.account.escrow.fetch(escrowPda[0]);
        const [onChainB1, onChainB2, onChainB3, onChainB4, onChainB5, onChainB6] = Array.from(
            [initialiserPda[0], bettor2Pda[0], bettor3Pda[0], bettor4Pda[0], bettor5Pda[0], bettor6Pda[0]],
            async (a) => {
                return await program.account.bettor.fetch(a)
            }
        );

        assert((await onChainB1).totAgainst.toNumber() + (await onChainB1).totFor.toNumber() == amount1);
        assert((await onChainB2).totAgainst.toNumber() + (await onChainB2).totFor.toNumber() == amount2);
        assert((await onChainB2).totUnderdog.toNumber() == 0);
        assert((await onChainB3).totAgainst.toNumber() + (await onChainB3).totFor.toNumber() == amount3);
        assert((await onChainB4).totAgainst.toNumber() + (await onChainB4).totFor.toNumber() == amount4);
        assert((await onChainB5).totAgainst.toNumber() + (await onChainB5).totFor.toNumber() == 0);
        assert((await onChainB5).totUnderdog.toNumber() == amount5);
        assert((await onChainB6).totAgainst.toNumber() + (await onChainB6).totFor.toNumber() == amount6);
        assert(onChainEscrow.totAgainst.toNumber() + onChainEscrow.totFor.toNumber() + onChainEscrow.totUnderdog.toNumber() == amount1 + amount2 + amount3 + amount4 + amount5 + amount6);


        // ----- EVALUATE ------

        assert(true);

    });

});
