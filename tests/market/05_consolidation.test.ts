import { readFileSync } from "fs";
import { homedir } from "os";
import path from "path";
import { Clock, type LiteSVM } from "litesvm";
import { fromWorkspace, LiteSVMProvider } from "anchor-litesvm";
import * as anchor from "@coral-xyz/anchor";
import { AnchorError, Program } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import nacl from "tweetnacl";
import naclUtil from "tweetnacl-util";

import {assert, expect } from "chai";

import { Market } from "../../target/types/market";
const MarketIDL = require("../../target/idl/market.json");
import { VotingTokens } from "../../target/types/voting_tokens";
const VotingTokensIDL = require("../../target/idl/voting_tokens.json");

describe("Consolidate", () => {

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

    // Facets
    const truthfulness = { truthfulness: {} };
    const originality = { originality: {} };
    const authenticity = { authenticity: {} };
    

    it("Consolidates bets and votes", async () => {

        // ------- SETUP -------

        const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

        const client = fromWorkspace("./");
        const provider = new LiteSVMProvider(client);
        const program = new Program<Market>(
            MarketIDL,
            provider,
        );
        const mintProgram = new Program<VotingTokens>(
            VotingTokensIDL,
            provider
        );
        client.addProgramFromFile(TOKEN_METADATA_PROGRAM_ID, ".anchor/metaplex.so");


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
            voter7,
        ] = Array.from({ length: 15 }, () => Keypair.generate());
        const treasury = loadKeypairFromFile("authensus_treasury_keypair.json");
        client.airdrop(treasury.publicKey, BigInt(1*LAMPORTS_PER_SOL));

        // Fetch mint pda and program id
        const mintfn = async () => {
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
                .rpc();
    
            return [mintPda[0], mintProgram.programId];
        }

        const [mintPda, mintProgramId] = await mintfn();
    
        // mint tokens to an ata
        const mintTo = async (
            {accounts, amount, payer}:
            {
                accounts: { payer: PublicKey; recipient: PublicKey; mint: PublicKey; recipient_ata: PublicKey; associated_token_program: PublicKey; system_program: PublicKey; token_program: PublicKey; rent: PublicKey; },
                amount: number,
                payer: Keypair
            }
        ) => {
    
            await mintProgram.methods.mintTokens(new anchor.BN(amount))
                .accounts({...accounts})
                .signers([payer])
                .rpc();

        }

        // Get voting token ATAs
        const treasuryAta = getAssociatedTokenAddressSync(mintPda, treasury.publicKey, true);
        const [
            voter1Ata,
            voter2Ata,
            voter3Ata,
            voter4Ata,
            voter5Ata,
            voter6Ata,
            voter7Ata,
            bettor1Ata,
            bettor2Ata,
            bettor3Ata,
            bettor4Ata,
            bettor5Ata,
            bettor6Ata,
        ] = Array.from(
            [voter1, voter2, voter3, voter4, voter5, voter6, voter7, firstBettor, bettor2, bettor3, bettor4, bettor5, bettor6],
            (v) => getAssociatedTokenAddressSync(mintPda, v.publicKey, true)
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
        const [voter1Pda, voter2Pda, voter3Pda, voter4Pda, voter5Pda, voter6Pda, voter7Pda] = Array.from(
            [voter1, voter2, voter3, voter4, voter5, voter6, voter7],
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
        // All the voters' PDAs for betting
        const [voter1wagerPda, voter2wagerPda, voter3wagerPda, voter4wagerPda, voter5wagerPda, voter6wagerPda, voter7wagerPda] = Array.from(
            [voter1, voter2, voter3, voter4, voter5, voter6, voter7],
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
        
        function simulateTimeTravel(svmClient: LiteSVM, secondsForward: number) {
            const slotsForward = Math.round((5*secondsForward)/2);  // 400ms per slot on avg
            const currentSlot = svmClient.getClock().slot;
            svmClient.warpToSlot(currentSlot + BigInt(slotsForward));
            svmClient.expireBlockhash();    // Move on to the next blockhash
            const cl: Clock = svmClient.getClock();
            const newTimestamp = cl.unixTimestamp + BigInt(secondsForward);
            svmClient.setClock(
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
        const [_airdrops] = Array.from(
            [initAdmin, firstBettor, bettor2, bettor3, bettor4, bettor5, bettor6, voter1, voter2, voter3, voter4, voter5, voter6, voter7],
            (s) => client.airdrop(s.publicKey, BigInt(1*LAMPORTS_PER_SOL))
        );

        // Mint 1SOL worth of tokens to voters
        const votersAndAtasList = [
            {v: treasury, ata: treasuryAta},
            {v: voter1, ata: voter1Ata},
            {v: voter2, ata: voter2Ata},
            {v: voter3, ata: voter3Ata},
            {v: voter4, ata: voter4Ata},
            {v: voter5, ata: voter5Ata},
            {v: voter6, ata: voter6Ata},
            {v: voter7, ata: voter7Ata},
        ];
        for (var index in votersAndAtasList) {
            
            const accounts = {
                payer: treasury.publicKey,
                recipient: votersAndAtasList[index].v.publicKey,
                mint: mintPda,
                recipient_ata: votersAndAtasList[index].ata,
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
                system_program: SystemProgram.programId,
                token_program: TOKEN_PROGRAM_ID,
                rent: SYSVAR_RENT_PUBKEY,
            };
            await mintTo({accounts: accounts, amount: 1*LAMPORTS_PER_SOL, payer: treasury})
        }

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
            voteAmount7,
        ] = Array.from(
            { length: 13 },
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
            voteDirection7,
        ] = Array.from(
            { length: 13 },
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

        const [voter1Accounts, voter2Accounts, voter3Accounts, voter4Accounts, voter5Accounts, voter6Accounts, voter7Accounts] = Array.from(
            [
                {a: voter1, pda: voter1Pda, betPda: voter1wagerPda, ata: voter1Ata},
                {a: voter2, pda: voter2Pda, betPda: voter2wagerPda, ata: voter2Ata},
                {a: voter3, pda: voter3Pda, betPda: voter3wagerPda, ata: voter3Ata},
                {a: voter4, pda: voter4Pda, betPda: voter4wagerPda, ata: voter4Ata},
                {a: voter5, pda: voter5Pda, betPda: voter5wagerPda, ata: voter5Ata},
                {a: voter6, pda: voter6Pda, betPda: voter6wagerPda, ata: voter6Ata},
                {a: voter7, pda: voter7Pda, betPda: voter7wagerPda, ata: voter7Ata},
            ],
            (obj) => {
                return {
                    signer: obj.a.publicKey,
                    treasury: treasury.publicKey,
                    market: marketPda[0],
                    poll: pollPda[0],
                    voter: obj.pda[0],
                    bettor: obj.betPda[0],
                    votingTokenAccount: obj.ata,
                    mint: mintPda,
                    treasuryVotingTokenAccount: treasuryAta,
                    systemProgram: SystemProgram.programId,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
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

        const [voterSignedMessage1, voterSignedMessage2, voterSignedMessage3, voterSignedMessage4, voterSignedMessage5, voterSignedMessage6, voterSignedMessage7] = Array.from(
            [
                {account: voter1, voteAmount: voteAmount1, dir: voteDirection1},
                {account: voter2, voteAmount: voteAmount2, dir: voteDirection2},
                {account: voter3, voteAmount: voteAmount3, dir: voteDirection3},
                {account: voter4, voteAmount: voteAmount4, dir: voteDirection4},
                {account: voter5, voteAmount: voteAmount5, dir: voteDirection5},
                {account: voter6, voteAmount: voteAmount6, dir: voteDirection6},
                {account: voter7, voteAmount: voteAmount7, dir: voteDirection7},
            ],
            (v) => {
                const dirString = v.dir ? "for" : "against";
                const str = authensusToken.toString() + "1" + "truthfulness" + v.account.publicKey.toString() + v.voteAmount.toString() + dirString;
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
        for (var index in betsObjList) {
            if (Math.random() > 0.05) {
                await program.methods.wager(marketParams, new anchor.BN(betsObjList[index].amt), betsObjList[index].dir, [...betsObjList[index].msg])
                    .accounts({ ...betsObjList[index].acc})
                    .signers([betsObjList[index].signer])
                    .rpc();
            } else {
                await program.methods.underdogBet(marketParams, new anchor.BN(betsObjList[index].amt), [...betsObjList[index].msg])
                    .accounts({ ...betsObjList[index].acc})
                    .signers([betsObjList[index].signer])
                    .rpc();
            }

            simulateTimeTravel(client, 1);
        }

        // Fast forward in time
        let secondsForwards = timeout + (5*60);   // 5 minutes after timeout
        simulateTimeTravel(client, secondsForwards);

        // Place votes
        const votesObjList = [
            {amt: voteAmount1, dir: voteDirection1, msg: voterSignedMessage1, acc: voter1Accounts, signer: voter1},
            {amt: voteAmount2, dir: voteDirection2, msg: voterSignedMessage2, acc: voter2Accounts, signer: voter2},
            {amt: voteAmount3, dir: voteDirection3, msg: voterSignedMessage3, acc: voter3Accounts, signer: voter3},
            {amt: voteAmount4, dir: voteDirection4, msg: voterSignedMessage4, acc: voter4Accounts, signer: voter4},
            {amt: voteAmount5, dir: voteDirection5, msg: voterSignedMessage5, acc: voter5Accounts, signer: voter5},
        ];

        for (var index in votesObjList) {
            await program.methods.vote(marketParams, new anchor.BN(votesObjList[index].amt), votesObjList[index].dir, [...votesObjList[index].msg])
                .accounts({ ...votesObjList[index].acc})
                .signers([votesObjList[index].signer, treasury])
                .rpc();

            simulateTimeTravel(client, 1);
        }

        // Fast forward in time again
        secondsForwards = 48*60*60 - (10*60);   // 5 minutes before voting timeout
        simulateTimeTravel(client, secondsForwards);

        // Make sure that the votes can still be made at this point
        await program.methods.vote(marketParams, new anchor.BN(voteAmount6), voteDirection6, [...voterSignedMessage6])
                .accounts({ ...voter6Accounts})
                .signers([voter6, treasury])
                .rpc();

        // Fast forward in time again
        secondsForwards = 20*60;   // 15 minutes after voting timeout
        simulateTimeTravel(client, secondsForwards);

        // Shouldn't be allowed to vote after timeout
        try {
            await program.methods.vote(marketParams, new anchor.BN(voteAmount7), voteDirection7, [...voterSignedMessage7])
                .accounts({ ...voter7Accounts})
                .signers([voter7, treasury])
                .rpc();
        
            assert(false, "Allowed vote after closing")
        } catch (err) {
            expect(err).to.be.instanceOf(AnchorError);
            expect((err as AnchorError).error.errorCode.number).to.equal(6707);
            expect((err as AnchorError).error.errorCode.code).to.equal("VotingClosed");
        }

        const consolidateBettor1accounts = {
            treasury: treasury.publicKey,
            signer: firstBettor.publicKey,
            market: marketPda[0],
            escrow: escrowPda[0],
            bettor: initialiserPda[0],
            poll: pollPda[0],
            mint: mintPda,
            recipient: bettor1Ata,
            votingTokensProgram: mintProgramId,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
        };

        const consolidateVoter1accounts = {
            treasury: treasury.publicKey,
            signer: voter1.publicKey,
            market: marketPda[0],
            poll: pollPda[0],
            voter: voter1Pda[0],
            votingTokenAccount: voter1Ata,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            mint: mintPda,
            systemProgram: SystemProgram.programId,
            votingTokensProgram: mintProgramId,
            rent: SYSVAR_RENT_PUBKEY,
        };

        // Start consolidations
        await program.methods.wagerResults(marketParams)
            .accounts({...consolidateBettor1accounts})
            .signers([firstBettor, treasury])
            .rpc();

        simulateTimeTravel(client, 1);

        await program.methods.voterResults(marketParams)
            .accounts({...consolidateVoter1accounts})
            .signers([voter1, treasury])
            .rpc();


        // ----- EVALUATE ------


        /*
        const onChainPoll = await program.account.poll.fetch(pollPda[0]);
        const [onChainV1, onChainV2, onChainV3, onChainV4, onChainV5, onChainV6] = Array.from(
            [voter1Pda[0], voter2Pda[0], voter3Pda[0], voter4Pda[0], voter5Pda[0], voter6Pda[0]],
            async (a) => {
                return await program.account.voter.fetch(a)
            }
        );

        assert((await onChainV1).amount.toNumber() == voteAmount1 && (await onChainV1).direction == voteDirection1);
        assert((await onChainV2).amount.toNumber() == voteAmount2 && (await onChainV2).direction == voteDirection2);
        assert((await onChainV3).amount.toNumber() == voteAmount3 && (await onChainV3).direction == voteDirection3);
        assert((await onChainV4).amount.toNumber() == voteAmount4 && (await onChainV4).direction == voteDirection4);
        assert((await onChainV5).amount.toNumber() == voteAmount5 && (await onChainV5).direction == voteDirection5);
        assert((await onChainV6).amount.toNumber() == voteAmount6 && (await onChainV6).direction == voteDirection6);

        const totFor = (voteDirection1 ? 1 : 0) + (voteDirection2 ? 1 : 0) + (voteDirection3 ? 1 : 0) + (voteDirection4 ? 1 : 0) + (voteDirection5 ? 1 : 0) + (voteDirection1 ? 1 : 0)
        const totAgainst = (voteDirection1 ? 0 : 1) + (voteDirection2 ? 0 : 1) + (voteDirection3 ? 0 : 1) + (voteDirection4 ? 0 : 1) + (voteDirection5 ? 0 : 1) + (voteDirection1 ? 0 : 1);
        
        assert(onChainPoll.totalFor == totFor);
        assert(onChainPoll.totalAgainst == totAgainst);
        */
        
    });

});
