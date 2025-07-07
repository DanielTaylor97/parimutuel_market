use anchor_lang::prelude::*;

use crate::states::{Escrow, Market, MarketParams, MarketState, Poll};
use crate::error::{FacetError, MarketError};

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct CallMarket<'info_c> {
    #[account(mut)]
    pub admin: Signer<'info_c>,
    #[account(
        mut,
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info_c, Market>>,
    #[account(
        mut,
        seeds = [b"poll", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Box<Account<'info_c, Poll>>,
    #[account(
        mut,
        seeds = [b"escrow", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Box<Account<'info_c, Escrow>>,
}

impl<'info_c> CallMarket<'info_c> {

    pub fn end(
        &mut self,
        params: &MarketParams,
    ) -> Result<()> {

        // Requirements:                                                        |   Implemented:
        //  - Market State should be Consolidating                              |       √
        //  - escrow and poll should have the same market, which is this market |       √
        //  - escrow/poll facet should be in the market facets vec              |       √
        require!(self.market.state == MarketState::Consolidating, MarketError::MarketInWrongState);
        require!(self.market.key() == self.escrow.market && self.market.key() == self.poll.market && self.market.token == params.authensus_token, MarketError::NotTheSameMarket);
        require!(self.market.facets.contains(&self.escrow.facet), FacetError::FacetNotInMarket);

        // Set market inactive
        self.market.state = MarketState::Inactive;

        // Empty escrow
        self.escrow.tot_for = 0_u64;
        self.escrow.tot_against = 0_u64;
        self.escrow.tot_underdog = 0_u64;

        // Empty poll
        self.poll.total_for = 0_u16;
        self.poll.total_against = 0_u16;

        Ok(())
    }

}
