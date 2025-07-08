#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

declare_id!("3ncdVr7aT7GBniBtcsgrjLvDsxXmBxRtk6TwVhMRHXo4");

pub mod states;
pub mod contexts;
pub mod utils;
pub mod error;
pub mod constants;

pub use states::*;
pub use contexts::*;
pub use utils::*;
pub use error::*;
pub use constants::*;

#[program]
pub mod market {
    use super::*;

    pub fn init_marketplace(ctx: Context<InitialiseMarketplace>) -> Result<()> {
        ctx.accounts.init_marketplace()
    }

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
        signed_message: [u8; 64],
    ) -> Result<()> {

        ctx.accounts.start(
            &ctx.bumps,
            &params,
        )?;

        ctx.accounts.first_bet(
            &ctx.bumps,
            &params,
            amount,
            direction,
            signed_message,
        )

    }

    pub fn wager(
        ctx: Context<Wager>,
        params: MarketParams,
        amount: u64,
        direction: bool,
        signed_message: [u8; 64],
    ) -> Result<()> {

        ctx.accounts.place_wager(
            &ctx.bumps,
            &params,
            amount,
            direction,
            signed_message,
        )

    }

    pub fn underdog_bet(
        ctx: Context<Wager>,
        params: MarketParams,
        amount: u64,
        signed_message: [u8; 64],
    ) -> Result<()> {

        ctx.accounts.underdog_bet(
            &ctx.bumps,
            &params,
            amount,
            signed_message,
        )
        
    }

    pub fn vote(
        ctx: Context<Vote>,
        params: MarketParams,
        amount: u64,
        direction: bool,
        signed_message: [u8; 64],
    ) -> Result<()> {

        ctx.accounts.add_vote(
            &ctx.bumps,
            &params,
            amount,
            direction,
            signed_message,
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
