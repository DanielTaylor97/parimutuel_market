#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

declare_id!("H4jYJQJhPSy7ANZwDZDkvE4Q9x5oQDz1tKaB2GRjrDpY");

pub mod states;
pub mod contexts;
pub mod error;
pub mod constants;

pub use states::*;
pub use contexts::*;
pub use error::*;
pub use constants::*;

#[program]
pub mod parimutuel_market {
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

    }

    pub fn start_market(
        ctx: Context<StartMarket>,
        params: MarketParams,
        amount: u64,
        direction: bool,
        timeout: i64,
    ) -> Result<()> {

        ctx.accounts.start(
            &ctx.bumps,
            &params,
            timeout,
        );

        ctx.accounts.first_bet(
            &ctx.bumps,
            params.address,
            amount,
            direction
        )

    }

    pub fn wager(
        ctx: Context<Wager>,
        params: MarketParams,
        amount: u64,
        direction: bool,
    ) -> Result<()> {

        ctx.accounts.place_wager(
            &ctx.bumps,
            &params,
            amount,
            direction,
        )

    }

    pub fn underdog_bet(
        ctx: Context<Wager>,
        params: MarketParams,
        amount: u64,
    ) -> Result<()> {

        ctx.accounts.underdog_bet(
            &ctx.bumps,
            &params,
            amount
        )
        
    }

    pub fn vote(
        ctx: Context<Vote>,
        params: MarketParams,
        amount: u64,
        direction: bool,
    ) -> Result<()> {

        ctx.accounts.add_vote(
            &ctx.bumps,
            &params,
            amount,
            direction
        )
        
    }

    pub fn voter_results(
        ctx: Context<VoterResult>,
        params: MarketParams,
    ) -> Result<()> {

        ctx.accounts.distribute_sol_to_voters(params.address)

    }

    pub fn wager_results(
        ctx: Context<WagerResult>,
        params: MarketParams,
    ) -> Result<()> {

        ctx.accounts.distribute_tokens_to_bettors_and_assign_markets(
            &params,
        )

    }

    pub fn call_market(
        ctx: Context<CallMarket>,
        authensus_token: Pubkey,
        facet: Facet,
    ) -> Result<()> {

        ctx.accounts.end(
            authensus_token,
            facet,
        )

    }
}
