use anchor_lang::prelude::*;

use crate::constants::VOTE_THRESHOLD;
use crate::error::VotingError;
use crate::states::{Bettor, Escrow, Facet, Market, MarketState, Poll, Voter};

#[derive(Accounts)]
#[instruction(authensus_token: Pubkey, facet: Facet)]
pub struct CallMarket<'info_c> {
    #[account(mut)]
    pub admin: Signer<'info_c>,
    #[account(
        seeds = [b"market", authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_c, Market>,
    #[account(
        seeds = [b"poll", authensus_token.as_ref(), facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Account<'info_c, Poll>,
    #[account(
        seeds = [b"escrow", authensus_token.as_ref(), facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Account<'info_c, Escrow>,
}

impl<'info_c> CallMarket<'info_c> {
    
    pub async fn determine_result(
        &mut self,
        program_id: &Pubkey,
        authensus_token: Pubkey,
        facet: Facet,
    ) -> Result<bool> {

        require!(self.poll.total_for + self.poll.total_against < VOTE_THRESHOLD, VotingError::NotVotingTime);   // Better to do time- or threshold-based?

        let mut tot_for: u64 = 0_u64;
        let mut tot_against: u64 = 0_u64;

        let facet_str: String = facet.to_string();

        let mut voter_key: Pubkey;
        let mut bump: u8;
        // let mut seeds: &[&[u8]];

        for key in self.poll.voters.as_ref().unwrap().iter() {

            let seeds: &[&[u8]; 4] = &[b"voter", authensus_token.as_ref(), facet_str.as_bytes(), key.as_ref()];
            (voter_key, bump) = Pubkey::find_program_address(seeds, program_id);

            //

        }

        Ok(true)

    }

    pub fn distribute_sol_to_voters(
        &mut self,
        results: bool,
    ) -> Result<()> {
        Ok(())
    }

    pub fn distribute_tokens_to_stakers(
        &mut self,
        results: bool,
    ) -> Result<()> {
        Ok(())
    }

    pub fn assign_voting_markets_to_new_stakers(
        &mut self,
        results: bool,
    ) -> Result<()> {
        Ok(())
    }

    pub fn end(
        &mut self,
        authensus_token: Pubkey,
        facet: Facet,
    ) -> Result<()> {

        // Set market inactive, set escrow counts and polls to zero if not already done

        Ok(())
    }

}
