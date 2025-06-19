use anchor_lang::prelude::*;

use crate::states::{Facet, Market, MarketState};

#[derive(Accounts)]
#[instruction(authensus_token: Pubkey)]
pub struct InitialiseMarket<'info_i> {
    #[account(mut)]
    admin: Signer<'info_i>,
    #[account(
        init,
        space = Market::INIT_SPACE,
        payer = admin,
        seeds = [b"market", authensus_token.as_ref()],
        bump,
    )]
    market: Account<'info_i, Market>,
    system_program: Program<'info_i, System>,
}

impl<'info_i> InitialiseMarket<'info_i> {

    pub fn init_market(
        &mut self,
        bumps: &InitialiseMarketBumps,
        token: Pubkey,
        facets: Vec<Facet>,
    ) -> Result<()> {
        
        // require!();

        self.market.set_inner(
            Market {
                bump: bumps.market,                 // u8
                token,                              // Pubkey
                facets,                             // Vec<Facet>
                state: MarketState::Initialised,    // MarketState
                round: 0_u16,                       // u16
            }
        );

        Ok(())

    }
}
