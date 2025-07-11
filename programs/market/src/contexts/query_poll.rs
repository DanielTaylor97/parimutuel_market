use anchor_lang::prelude::*;

use crate::states::{Market, MarketParams, MarketState, Poll};
use crate::error::{FacetError, MarketError};

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct QueryPoll<'info_qp> {
    #[account(mut)]
    pub signer: Signer<'info_qp>,
    #[account(
        mut,
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info_qp, Market>>,
    #[account(
        init_if_needed,
        space = 8 + Poll::INIT_SPACE,
        payer = signer,
        seeds = [b"poll", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Box<Account<'info_qp, Poll>>,
    pub system_program: Program<'info_qp, System>,
}

impl<'info_qp> QueryPoll<'info_qp> {

    pub fn query_poll(&mut self, params: &MarketParams) -> Result<u16> {

        // Requirements:                            |   Implemented:
        //  - Facet should exist in the market      |       √
        //  - Market state should be Consolidating  |       √
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.market.state == MarketState::Consolidating, MarketError::MarketInWrongState);

        Ok((100*self.poll.total_for)/(self.poll.total_for + self.poll.total_against))

    }

}