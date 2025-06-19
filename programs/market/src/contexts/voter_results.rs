use anchor_lang::{
    prelude::*,
    system_program::{Transfer, transfer}
};

use crate::constants::VOTE_THRESHOLD;
use crate::error::ResultsError;
use crate::states::{Escrow, Facet, Market, MarketState, Poll, Voter};

#[derive(Accounts)]
#[instruction(authensus_token: Pubkey, facet: Facet, address: Pubkey)]
pub struct VoterResult<'info_r> {
    #[account(mut)]
    pub signer: Signer<'info_r>,
    #[account(
        seeds = [b"market", authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_r, Market>,
    // #[account(
    //     seeds = [b"escrow", authensus_token.as_ref(), facet.to_string().as_bytes()],
    //     bump,
    // )]
    // pub escrow: Account<'info_r, Escrow>,
    #[account(
        seeds = [b"poll", authensus_token.as_ref(), facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Account<'info_r, Poll>,
    #[account(
        seeds = [b"voter", authensus_token.as_ref(), facet.to_string().as_bytes(), address.as_ref()],
        bump,
    )]
    pub voter: Account<'info_r, Voter>,
}

impl<'info_r> VoterResult<'info_r> {

    pub fn distribute_sol_to_voters(&mut self) -> Result<()> {

        require!(self.market.state == MarketState::Inactive, ResultsError::VotingNotFinished);
        // require!(self.escrow.total_underdog == 0, ResultsError::UnderdogBetsNotResolved);

        Ok(())

    }

}
