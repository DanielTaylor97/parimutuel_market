import { readFileSync } from "fs";
import { homedir } from "os";
import path from "path";
import { Clock, type ProgramTestContext, startAnchor } from "solana-bankrun";
import { BankrunProvider } from "anchor-bankrun";
import * as anchor from "@coral-xyz/anchor";
import { AnchorError, Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction, type GetVersionedTransactionConfig } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import nacl from "tweetnacl";
import naclUtil from "tweetnacl-util";

import {assert, expect } from "chai";

import { Market } from "../../target/types/market";
// import * as MarketIDL from "../../target/idl/market.json";
const MarketIDL = require("../../target/idl/market.json");
import { VotingTokens } from "../../target/types/voting_tokens";

describe("Place Votes", () => {

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

    const [mintPda, _mintProgramId] = await mintfn();

    // Facets
    const truthfulness = { truthfulness: {} };
    const originality = { originality: {} };
    const authenticity = { authenticity: {} };

    const [
        authensusTokenKP,
        initAdmin,
        firstBettor,
        bettor2,
        bettor3,
        bettor4,
        bettor5,
        bettor6,
        voter1,
        voter2,
        voter3,
        voter4,
        voter5,
        voter6,
    ] = Array.from({ length: 14 }, () => Keypair.generate());
    const treasury = loadKeypairFromFile("authensus_treasury_keypair.json");

    // Get voting token ATAs
    const treasuryAta = getAssociatedTokenAddressSync(mintPda, treasury.publicKey, true);
    const [
        voter1Ata,
        voter2Ata,
        voter3Ata,
        voter4Ata,
        voter5Ata,
        voter6Ata,
    ] = Array.from(
        [voter1, voter2, voter3, voter4, voter5, voter6],
        (v) => getAssociatedTokenAddressSync(mintPda, v.publicKey, true)
    );
    

    it("Places votes", async () => {

        // ------- SETUP -------

        const context = await startAnchor("./", [], []);
        console.log(`Reached line 60`)
        const provider = new BankrunProvider(context);
        const program = new Program<Market>(
            MarketIDL,
            provider,
        );

        // PDA for the market
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
        // All the voters' PDAs for voting
        const [voter1Pda, voter2Pda, voter3Pda, voter4Pda, voter5Pda, voter6Pda] = Array.from(
            [voter1, voter2, voter3, voter4, voter5, voter6],
            (v) => PublicKey.findProgramAddressSync(
                [
                    Buffer.from("voter"),
                    authensusTokenKP.publicKey.toBuffer(),
                    Buffer.from("truthfulness"),
                    v.publicKey.toBuffer(),
                ],
                program.programId
            )
        );
        // All the voters' PDAs for voting
        const [voter1wagerPda, voter2wagerPda, voter3wagerPda, voter4wagerPda, voter5wagerPda, voter6wagerPda] = Array.from(
            [voter1, voter2, voter3, voter4, voter5, voter6],
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
        
        async function simulateTimeTravel(ctx: ProgramTestContext, secondsForward: number) {
            const cl: Clock = await ctx.banksClient.getClock();
            const newTimestamp = cl.unixTimestamp + BigInt(secondsForward);
            ctx.setClock(
                new Clock(
                    cl.slot,
                    cl.epochStartTimestamp,
                    cl.epoch,
                    cl.leaderScheduleEpoch,
                    newTimestamp,
                )
            );
        }

        // Airdrop SOL
        let airdropTx = new Transaction();
        airdropTx.instructions = Array.from(
            [initAdmin, firstBettor, bettor2, bettor3, bettor4, bettor5, bettor6, voter1, voter2, voter3, voter4, voter5, voter6],
            (s) => SystemProgram.transfer({
                fromPubkey: provider.publicKey,
                toPubkey: s.publicKey,
                lamports: 1*LAMPORTS_PER_SOL,
            })
        );
        await provider.sendAndConfirm(airdropTx, []);

        // MINT TOKENS TO VOTERS!!!

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
            .rpc();

        const marketParams = {
            authensusToken: authensusToken,
            facet: truthfulness,
        };
        const [
            betAmount1,
            betAmount2,
            betAmount3,
            betAmount4,
            betAmount5,
            betAmount6,
            voteAmount1,
            voteAmount2,
            voteAmount3,
            voteAmount4,
            voteAmount5,
            voteAmount6,
        ] = Array.from(
            { length: 6 },
            () => Math.round(Math.random()*100_000)*1_000
        );
        const [
            betDirection1,
            betDirection2,
            betDirection3,
            betDirection4,
            betDirection5,
            betDirection6,
            voteDirection1,
            voteDirection2,
            voteDirection3,
            voteDirection4,
            voteDirection5,
            voteDirection6,
        ] = Array.from(
            { length: 6 },
            () => Math.random() > 0.2 // 80% betting on true on avg
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

        const [voter1Accounts, voter2Accounts, voter3Accounts, voter4Accounts, voter5Accounts, voter6Accounts] = Array.from(
            [
                {a: voter1, pda: voter1Pda, betPda: voter1wagerPda, ata: voter1Ata},
                {a: voter2, pda: voter2Pda, betPda: voter2wagerPda, ata: voter2Ata},
                {a: voter3, pda: voter3Pda, betPda: voter3wagerPda, ata: voter3Ata},
                {a: voter4, pda: voter4Pda, betPda: voter4wagerPda, ata: voter4Ata},
                {a: voter5, pda: voter5Pda, betPda: voter5wagerPda, ata: voter5Ata},
                {a: voter6, pda: voter6Pda, betPda: voter6wagerPda, ata: voter6Ata},
            ],
            (obj) => {
                return {
                    signer: obj.a.publicKey,
                    treasury: treasury.publicKey,
                    market: marketPda[0],
                    poll: pollPda[0],
                    voter: obj.pda[0],
                    bettor: obj.betPda[0],
                    voting_token_account: obj.ata,
                    mint: mintPda,
                    treasury_voting_token_account: treasuryAta,
                    system_program: SystemProgram.programId,
                    token_program: TOKEN_PROGRAM_ID,
                    associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
                }
            }
        );

        const [bettorSignedMessage1, bettorSignedMessage2, bettorSignedMessage3, bettorSignedMessage4, bettorSignedMessage5, bettorSignedMessage6] = Array.from(
            [
                {account: firstBettor, betAmount: betAmount1},
                {account: bettor2, betAmount: betAmount2},
                {account: bettor3, betAmount: betAmount3},
                {account: bettor4, betAmount: betAmount4},
                {account: bettor5, betAmount: betAmount5},
                {account: bettor6, betAmount: betAmount6},
            ],
            (b) => {
                const str = authensusToken.toString() + "1" + "truthfulness" + b.account.publicKey.toString() + b.betAmount.toString();
                const enc = naclUtil.decodeUTF8(str);
                return nacl.sign.detached(enc, treasury.secretKey)
            }
        );

        const [voterSignedMessage1, voterSignedMessage2, voterSignedMessage3, voterSignedMessage4, voterSignedMessage5, voterSignedMessage6] = Array.from(
            [
                {account: voter1, voteAmount: voteAmount1, dir: voteDirection1},
                {account: voter2, voteAmount: voteAmount2, dir: voteDirection2},
                {account: voter3, voteAmount: voteAmount3, dir: voteDirection3},
                {account: voter4, voteAmount: voteAmount4, dir: voteDirection4},
                {account: voter5, voteAmount: voteAmount5, dir: voteDirection5},
                {account: voter6, voteAmount: voteAmount6, dir: voteDirection6},
            ],
            (b) => {
                const dirString = b.dir ? "for" : "against";
                const str = authensusToken.toString() + "1" + "truthfulness" + b.account.publicKey.toString() + b.voteAmount.toString() + dirString;
                const enc = naclUtil.decodeUTF8(str);
                return nacl.sign.detached(enc, treasury.secretKey)
            }
        );

        // Start the market
        await program.methods.startMarket(marketParams, new anchor.BN(betAmount1), betDirection1, [...bettorSignedMessage1])
            .accounts({ ...startMarketAccounts })
            .signers([firstBettor])
            .rpc();
        
        // Place bets
        const betsObjList = [
            {amt: betAmount2, dir: betDirection2, msg: bettorSignedMessage2, acc: bettor2Accounts, signer: bettor2},
            {amt: betAmount3, dir: betDirection3, msg: bettorSignedMessage3, acc: bettor3Accounts, signer: bettor3},
            {amt: betAmount4, dir: betDirection4, msg: bettorSignedMessage4, acc: bettor4Accounts, signer: bettor4},
            {amt: betAmount5, dir: betDirection5, msg: bettorSignedMessage5, acc: bettor5Accounts, signer: bettor5},
            {amt: betAmount6, dir: betDirection6, msg: bettorSignedMessage6, acc: bettor6Accounts, signer: bettor6},
        ];
        betsObjList.forEach( async (obj) => {
            if (Math.random() > 0.05) {
                await program.methods.wager(marketParams, new anchor.BN(obj.amt), obj.dir, [...obj.msg])
                    .accounts({ ...obj.acc})
                    .signers([obj.signer])
                    .rpc();
            } else {
                await program.methods.underdogBet(marketParams, new anchor.BN(obj.amt), [...obj.msg])
                    .accounts({ ...obj.acc})
                    .signers([obj.signer])
                    .rpc();
            }
        })

        // Should throw the expected error when someone tries to place a vote before betting is finished
        try {
            await program.methods.vote(marketParams, new anchor.BN(voteAmount1), voteDirection1, [...voterSignedMessage1])
                .accounts({ ...voter1Accounts})
                .signers([voter1])
                .rpc();
        
            assert(false, "Allowed vote before betting finished")
        } catch (err) {
            expect(err).to.be.instanceOf(AnchorError);
            expect((err as AnchorError).error.errorCode.number).to.equal(6701);
            expect((err as AnchorError).error.errorCode.code).to.equal("NotVotingTime");
        }

        // Fast forward in time
        const secondsForwards = timeout - (5*60);   // 5 minutes before timeout
        simulateTimeTravel(context, secondsForwards);

        // Still shouldn't be allowed to place a vote
        try {
            await program.methods.vote(marketParams, new anchor.BN(voteAmount1), voteDirection1, [...voterSignedMessage1])
                .accounts({ ...voter1Accounts})
                .signers([voter1])
                .rpc();
        
            assert(false, "Allowed vote before betting finished")
        } catch (err) {
            expect(err).to.be.instanceOf(AnchorError);
            expect((err as AnchorError).error.errorCode.number).to.equal(6701);
            expect((err as AnchorError).error.errorCode.code).to.equal("NotVotingTime");
        }

        // Fast forward in time again
        const skipOverTimeout = 10*60;   // 10 more minutes -- now past betting window
        simulateTimeTravel(context, secondsForwards);

        await program.methods.vote(marketParams, new anchor.BN(voteAmount1), voteDirection1, [...voterSignedMessage1])
            .accounts({ ...voter1Accounts})
            .signers([voter1])
            .rpc();


        // ----- EVALUATE ------

        /*
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
        */

        assert(true);

    });

});
