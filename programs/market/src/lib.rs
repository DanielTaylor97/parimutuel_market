use anchor_lang::prelude::*;

declare_id!("H4jYJQJhPSy7ANZwDZDkvE4Q9x5oQDz1tKaB2GRjrDpY");

pub mod states;
pub mod contexts;
pub mod error;
pub mod constants;

pub use states::Facet;
pub use contexts::*;
pub use error::*;
pub use constants::*;

#[program]
pub mod market {
    use super::*;

    pub fn initialise_market(
        ctx: Context<InitialiseMarket>,
        authensus_token: Pubkey,
        facets: Vec<Facet>,
    ) -> Result<()> {

        ctx.accounts.init_market(
            &ctx.bumps,
            authensus_token,
            facets
        )
        // init_market(
        //     &ctx
        // )?;

        // Ok(())

    }

    pub fn start_market(
        ctx: Context<StartMarket>,
        address: Pubkey,
        authensus_token: Pubkey,
        facet: Facet,
        amount: u64,
        direction: bool,
        timeout: i64,
    ) -> Result<()> {

        ctx.accounts.start(
            &ctx.bumps,
            address,
            authensus_token,
            facet,
            timeout,
        );

        ctx.accounts.first_bet(
            address,
            amount,
            direction
        )
        // start(
        //     &ctx,
        //     authensus_token,
        // )?;

        // Ok(())

    }

    pub fn wager(
        ctx: Context<Wager>,
        address: Pubkey,
        authensus_token: Pubkey,
        facet: Facet,
        amount: u64,
        direction: bool,
    ) -> Result<()> {

        ctx.accounts.place_wager(
            address,
            authensus_token,
            facet,
            amount,
            direction,
        )

        // add_wager(
        //     &ctx,
        //     facet,
        //     amount,
        //     direction,
        // )?;

        // Ok(())
    }

    pub fn underdog_bet(
        ctx: Context<Wager>,
        address: Pubkey,
        authensus_token: Pubkey,
        facet: Facet,
        amount: u64,
    ) -> Result<()> {

        ctx.accounts.underdog_bet(
            address,
            authensus_token,
            facet,
            amount
        )
        // add_underdog_bet(
        //     &ctx,
        //     facet,
        //     amount,
        // )?;

        // Ok(())
    }

    pub fn vote(
        ctx: Context<Vote>,
        address: Pubkey,
        authensus_token: Pubkey,
        facet: Facet,
        amount: u64,
        direction: bool,
    ) -> Result<()> {

        ctx.accounts.add_vote(
            &ctx.bumps,
            address,
            authensus_token,
            facet,
            amount,
            direction
        )

        // add_vote(
        //     &ctx,
        //     facet,
        //     amount,
        //     direction,
        // )?;

        // Ok(())
    }

    pub fn voting_results(
        ctx: Context<VotingResult>,
        address: Pubkey,
        authensus_token: Pubkey,
        facet: Facet,
    ) -> Result<()> {

        ctx.accounts.distribute_sol_to_voters()

        // Ok(())

    }

    pub fn wager_results(
        ctx: Context<WagerResult>,
        address: Pubkey,
        authensus_token: Pubkey,
        facet: Facet,
    ) -> Result<()> {

        ctx.accounts.distribute_tokens_to_bettors_and_assign_markets(
            address,
            authensus_token,
            facet,
        )

        // Ok(())

    }

    pub fn call_market(
        ctx: Context<CallMarket>,
        authensus_token: Pubkey,
        facet: Facet,
    ) -> Result<()> {

        // // Also has to calculate underdog bets
        // let results = ctx.accounts.determine_result(
        //     authensus_token,
        //     facet
        // )?;

        // ctx.accounts.distribute_sol_to_voters(
        //     results,
        // )?;

        // ctx.accounts.distribute_tokens_to_stakers(
        //     results,
        // )?;

        // ctx.accounts.assign_voting_markets_to_new_stakers(
        //     results,
        // )?;

        ctx.accounts.end(
            authensus_token,
            facet,
        )

        // Ok(())

    }
}
