import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TreasuryProgram } from "../target/types/treasury";

describe("market", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.Market as Program<TreasuryProgram>;

    it("Is initialized!", async () => {

        // ------- SETUP -------

        //


        // ------ EXECUTE ------

        const tx = await program.methods.initialise()
            .accounts({ ...accounts })
            .signers([initializer])
            .rpc()
            .then(confirm)
            .then(log);


        // ----- EVALUATE ------

        console.log("Your transaction signature", tx);
    });
});
