use anchor_lang::prelude::*;

use crate::states::{Escrow, Market, MarketParams, MarketState, Poll};
use crate::constants::TREASURY_AUTHORITY;
use crate::error::{FacetError, MarketError, ResultsError, TreasuryError};
use crate::utils::functions::vec_eq;

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

        if self.escrow.bettors.is_none() || self.escrow.bettors_consolidated.is_none() || self.poll.voters.is_none() || self.poll.voters_consolidated.is_none() {
            return Err(anchor_lang::error!(ResultsError::NotAllBetsConsolidated))
        }

        let bet_consolidation: bool = vec_eq(self.escrow.bettors.clone().as_mut().unwrap(), self.escrow.bettors_consolidated.clone().as_mut().unwrap());
        let vote_consolidation: bool = vec_eq(self.poll.voters.clone().as_mut().unwrap(), self.poll.voters_consolidated.clone().as_mut().unwrap());

        // Requirements:                                                        |   Implemented:
        //  - Market State should be Consolidating                              |       √
        //  - escrow and poll should have the same market, which is this market |       √
        //  - escrow and poll should have the same facet                        |       √
        //  - escrow/poll facet should be in the market facets vec              |       √
        //  - SOL has been reimbursed as necessary                              |       √
        //  - Tokens have been reimbursed as necessary                          |       √
        //  - Admin should be the treasury authority                            |       √
        require!(self.market.state == MarketState::Consolidating, MarketError::MarketInWrongState);
        require!(self.market.key() == self.escrow.market && self.market.key() == self.poll.market && self.market.token == params.authensus_token, MarketError::NotTheSameMarket);
        require!(self.escrow.facet == self.poll.facet && self.escrow.facet == params.facet, FacetError::NotTheSameFacet);
        require!(self.market.facets.contains(&self.escrow.facet), FacetError::FacetNotInMarket);
        require!(bet_consolidation, ResultsError::NotAllBetsConsolidated);
        require!(vote_consolidation, ResultsError::NotAllVotesConsolidated);
        require!(self.admin.key().to_string() == TREASURY_AUTHORITY, TreasuryError::WrongTreasuryAuthority);

        // Set market inactive
        self.market.state = MarketState::Inactive;

        // Empty escrow
        self.escrow.bettors = None;
        self.escrow.bettors_consolidated = None;
        self.escrow.tot_for = 0_u64;
        self.escrow.tot_against = 0_u64;
        self.escrow.tot_underdog = 0_u64;

        // Empty poll
        self.poll.voters = None;
        self.poll.voters_consolidated = None;
        self.poll.total_for = 0_u64;
        self.poll.total_against = 0_u64;

        Ok(())
    }

}
