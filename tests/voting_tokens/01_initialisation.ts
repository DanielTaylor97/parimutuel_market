import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
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
    const mint = PublicKey.findProgramAddressSync(
      [Buffer.from(MINT_SEED)],
      program.programId
    )[0];
    const metadata = PublicKey.findProgramAddressSync(
        [
            Buffer.from(METADATA_SEED),
            TOKEN_METADATA_PROGRAM_ID.toBuffer(),
            mint.toBuffer(),
        ],
        program.programId
      )[0];
  
    const init_accounts = {
      signer: signer.publicKey,
      mint,
      metadata,
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
