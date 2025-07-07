use anchor_lang::prelude::*;

use crate::states::{Escrow, Market, MarketParams, MarketState, Poll};
use crate::error::MarketError;

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
        _params: &MarketParams,
    ) -> Result<()> {

        let time: i64 = Clock::get()?.unix_timestamp;

        let end_condition: bool;
        if let Some(call_time) = self.market.call_time {
            end_condition = (self.escrow.bets_consolidated == self.escrow.n_bets && self.poll.total_consolidated == self.poll.total_for + self.poll.total_against) || call_time < time;
        } else {
            end_condition = false;  // Definitely don't end the market if the call time doesn't have an i64 value
        }

        // Requirements:                                                                |   Implemented:
        //  - Market State should be Consolidating                                      |       √
        //  - All bets and votes must be reimbursed OR the call_time has been reached   |       √
        require!(self.market.state == MarketState::Consolidating, MarketError::MarketInWrongState);
        require!(end_condition, MarketError::MarketInWrongState);

        // Set market inactive
        self.market.state = MarketState::Inactive;

        // Empty escrow
        self.escrow.n_bets = 0_u16;
        self.escrow.bets_consolidated = 0_u16;
        self.escrow.tot_for = 0_u64;
        self.escrow.tot_against = 0_u64;
        self.escrow.tot_underdog = 0_u64;

        // Empty poll
        self.poll.total_for = 0_u16;
        self.poll.total_against = 0_u16;
        self.poll.total_consolidated = 0_u16;

        Ok(())
    }

}
