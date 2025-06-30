use anchor_lang::prelude::*;

use crate::states::{Escrow, Market, MarketParams, MarketState, Poll};
use crate::constants::TREASURY_AUTHORITY;
use crate::error::{FacetError, MarketError, ResultsError, TreasuryError};

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct CallMarket<'info_c> {
    #[account(mut)]
    pub admin: Signer<'info_c>,
    #[account(
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_c, Market>,
    #[account(
        seeds = [b"poll", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Account<'info_c, Poll>,
    #[account(
        seeds = [b"escrow", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Account<'info_c, Escrow>,
}

impl<'info_c> CallMarket<'info_c> {

    pub fn end(
        &mut self,
        params: &MarketParams,
    ) -> Result<()> {

        if self.escrow.bettors.is_none() || self.escrow.bettors_consolidated.is_none() || self.poll.voters.is_none() || self.poll.voters_consolidated.is_none() {
            return Err(anchor_lang::error!(ResultsError::NotAllBetsConsolidated))
        }

        let bet_consolidation: bool = self.vec_eq(self.escrow.bettors.clone().as_mut().unwrap(), self.escrow.bettors_consolidated.clone().as_mut().unwrap());
        let vote_consolidation: bool = self.vec_eq(self.poll.voters.clone().as_mut().unwrap(), self.poll.voters_consolidated.clone().as_mut().unwrap());

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

        self.market.set_inner(
            Market {
                bump: self.market.bump,             // u8
                token: self.market.token,           // Pubkey
                facets: self.market.facets.clone(), // Vec<Facet>
                start_time: self.market.start_time, // i64
                timeout: self.market.timeout,       // i64
                state: MarketState::Inactive,       // MarketState
                round: self.market.round,           // u16
            }
        );

        self.escrow.set_inner(
            Escrow{
                bump: self.escrow.bump,                 // u8
                initialiser: self.escrow.initialiser,   // Pubkey
                market: self.escrow.market,             // Pubkey
                facet: self.escrow.facet.clone(),       // Facet
                bettors: None,                          // Option<Vec<Pubkey>>
                bettors_consolidated: None,             // Option<Vec<Pubkey>>
                tot_for: 0_u64,                         // u64
                tot_against: 0_u64,                     // u64
                tot_underdog: 0_u64,                    // u64
            }
        );

        self.poll.set_inner(
            Poll{
                bump: self.poll.bump,           // u8
                market: self.poll.market,       // Pubkey
                facet: self.poll.facet.clone(), // Facet
                voters: None,                   // Option<Vec<Pubkey>>
                voters_consolidated: None,      // Option<Vec<Pubkey>>
                total_for: 0_u64,               // u64
                total_against: 0_u64,           // u64
            }
        );

        Ok(())
    }

    fn vec_eq(
        &mut self,
        v1: &mut Vec<Pubkey>,
        v2: &mut Vec<Pubkey>,
    ) -> bool {

        v1.sort();
        v2.sort();

        v1 == v2

    }

}
