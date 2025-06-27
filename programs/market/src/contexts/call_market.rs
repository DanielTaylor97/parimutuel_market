use anchor_lang::prelude::*;

use crate::error::{FacetError, MarketError};
use crate::states::{Escrow, Market, MarketParams, MarketState, Poll};

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

        // Requirements:
        //  - escrow and poll should have the same market, which is this market
        //  - escrow and poll should have the same facet
        //  - escrow/poll facet should be in the market facets vec
        //  - Tokens and SOL have been reimbursed as necessary                      TODO
        require!(self.market.key() == self.escrow.market && self.market.key() == self.poll.market && self.market.token == params.authensus_token, MarketError::NotTheSameMarket);
        require!(self.escrow.facet == self.poll.facet && self.escrow.facet == params.facet, FacetError::NotTheSameFacet);
        require!(self.market.facets.contains(&self.escrow.facet), FacetError::FacetNotInMarket);

        // Set market inactive if that hasn't already been done, set escrow counts and polls to zero if not already done

        if self.market.state != MarketState::Inactive {
            self.market.set_inner(
                Market {
                    bump: self.market.bump,             // u8
                    token: self.market.token,           // Pubkey
                    facets: self.market.facets.clone(), // Vec<Facet>
                    state: MarketState::Inactive,       // MarketState
                    round: self.market.round,           // u16
                }
            );
        }

        self.escrow.set_inner(
            Escrow{
                bump: self.escrow.bump,                 // u8
                initialiser: self.escrow.initialiser,   // Pubkey
                market: self.escrow.market,             // Pubkey
                facet: self.escrow.facet.clone(),       // Facet
                bettors: None,                          // Option<Vec<Pubkey>>
                start_time: self.escrow.start_time,     // i64
                end_time: self.escrow.end_time,         // i64
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
                total_for: 0_u64,               // u64
                total_against: 0_u64,           // u64
            }
        );

        Ok(())
    }

}
