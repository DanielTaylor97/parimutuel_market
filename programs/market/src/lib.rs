#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

use voting_tokens::{cpi::accounts::MintTokens, program::VotingTokens,};

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

    pub fn voting_results(
        ctx: Context<VotingResult>,
        params: MarketParams,
    ) -> Result<()> {

        ctx.accounts.distribute_sol_to_voters()

    }

    pub fn wager_results(
        ctx: Context<WagerResult>,
        ctx_cpi: Context<MintCpiStruct>,
        params: MarketParams,
    ) -> Result<()> {

        let program_account: AccountInfo<'_> = ctx_cpi.accounts.callee.to_account_info();
        let mint_accounts = MintTokens{
            payer: ctx_cpi.accounts.payer.to_account_info(),
            mint: ctx_cpi.accounts.mint.to_account_info(),
            recipient: ctx_cpi.accounts.recipient.to_account_info(),
            associated_token_program: ctx_cpi.accounts.associated_token_program.to_account_info(),
            system_program: ctx_cpi.accounts.system_program.to_account_info(),
            token_program: ctx_cpi.accounts.token_program.to_account_info(),
            rent: ctx_cpi.accounts.rent.to_account_info(),
        };

        ctx.accounts.distribute_tokens_to_bettors_and_assign_markets(
            &params,
            program_account,
            mint_accounts,
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

#[derive(Accounts)]
pub struct MintCpiStruct<'info_c> {
    #[account(mut)]
    pub payer: UncheckedAccount<'info_c>,   // Check that this works as expected since it is meant to be a signer on the other end
    #[account(mut)]
    pub mint: UncheckedAccount<'info_c>,
    #[account(mut)]
    pub recipient: UncheckedAccount<'info_c>,
    pub associated_token_program: UncheckedAccount<'info_c>,
    pub system_program: UncheckedAccount<'info_c>,
    pub token_program: UncheckedAccount<'info_c>,
    pub rent: UncheckedAccount<'info_c>,
    pub callee: Program<'info_c, VotingTokens>,
}
