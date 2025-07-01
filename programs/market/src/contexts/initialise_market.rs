use anchor_lang::prelude::*;

use crate::states::{Facet, Market, MarketState};
use crate::constants::{MIN_ALLOWED_TIMEOUT, MAX_ALLOWED_TIMEOUT};
use crate::error::InitError;

#[derive(Accounts)]
#[instruction(authensus_token: Pubkey)]
pub struct InitialiseMarket<'info_i> {
    #[account(mut)]
    pub admin: Signer<'info_i>,
    #[account(
        init,
        space = Market::INIT_SPACE,
        payer = admin,
        seeds = [b"market", authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_i, Market>,
    pub system_program: Program<'info_i, System>,
}

impl<'info_i> InitialiseMarket<'info_i> {

    pub fn init_market(
        &mut self,
        bumps: &InitialiseMarketBumps,
        token: Pubkey,
        facets: Vec<Facet>,
        timeout: i64,
    ) -> Result<()> {
        
        // Requirements:            |   Implemented:
        //  - At least one facet    |       √
        //  - Timeout not too large |       √
        //  - Timeout not too small |       √
        require!(facets.len() >= 1, InitError::NoFacetsProvided);
        require!(timeout <= MAX_ALLOWED_TIMEOUT, InitError::TimeoutTooLarge);
        require!(timeout >= MIN_ALLOWED_TIMEOUT, InitError::TimeoutTooSmall);

        self.market.set_inner(
            Market {
                bump: bumps.market,                 // u8
                token,                              // Pubkey
                facets,                             // Vec<Facet>
                start_time: 0_i64,                  // i64
                timeout,                            // i64
                state: MarketState::Initialised,    // MarketState
                round: 0_u16,                       // u16
            }
        );

        Ok(())

    }
}
