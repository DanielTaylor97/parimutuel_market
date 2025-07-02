import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { VotingTokens } from "../target/types/voting_tokens";

describe("market", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.Market as Program<VotingTokens>;

    it("Is initialized!", async () => {


        // ------- SETUP -------

        const params = {
            name: "",
            symbol: "",
            uri: "",
            decimals: 9,
        };


        // ------ EXECUTE ------

        const tx = await program.methods.init(params)
            .accounts({ ...accounts })
            .signers([initializer])
            .rpc()
            .then(confirm)
            .then(log);


        // ----- EVALUATE ------

        console.log("Your transaction signature", tx);

    });
});
