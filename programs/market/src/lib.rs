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
        timeout: i64,
    ) -> Result<()> {

        ctx.accounts.init_market(
            &ctx.bumps,
            authensus_token,
            facets,
            timeout,
        )

    }

    pub fn start_market(
        ctx: Context<StartMarket>,
        params: MarketParams,
        amount: u64,
        direction: bool,
    ) -> Result<()> {

        ctx.accounts.start(
            &ctx.bumps,
            &params,
        )?;

        ctx.accounts.first_bet(
            &ctx.bumps,
            &params,
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

        ctx.accounts.distribute_sol_to_voter(&params)

    }

    pub fn wager_results(
        ctx: Context<WagerResult>,
        params: MarketParams,
    ) -> Result<()> {

        ctx.accounts.assign_tokens_and_markets_to_bettor(&params)

    }

    pub fn call_market(
        ctx: Context<CallMarket>,
        params: MarketParams,
    ) -> Result<()> {

        ctx.accounts.end(
            &params,
        )

    }
}
