use anchor_lang::prelude::*;

use crate::states::{Escrow, Market, MarketParams};
use crate::error::FacetError;

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct QueryEscrow<'info_qe> {
    #[account(mut)]
    pub signer: Signer<'info_qe>,
    #[account(
        mut,
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info_qe, Market>>,
    #[account(
        init_if_needed, // May in theory not exist, in which case it needs to be initialised
        space = 8 + Escrow::INIT_SPACE,
        payer = signer,
        seeds = [b"escrow", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Box<Account<'info_qe, Escrow>>,
    pub system_program: Program<'info_qe, System>,
}

impl<'info_qe> QueryEscrow<'info_qe> {

    pub fn query_escrow(&mut self, params: &MarketParams) -> Result<u64> {

        // Requirements:                        |   Implemented:
        //  - Facet should exist in the market  |       √
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);

        Ok(self.escrow.tot_for + self.escrow.tot_against + self.escrow.tot_underdog)

    }

}