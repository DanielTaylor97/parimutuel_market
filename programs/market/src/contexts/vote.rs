use anchor_lang::{
    prelude::*,
    system_program::{Transfer, transfer}
};

use crate::constants::VOTE_THRESHOLD;
use crate::error::VotingError;
use crate::states::{Escrow, Market, MarketParams, MarketState, Poll, Voter};

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct Vote<'info_v> {
    #[account(mut)]
    pub signer: Signer<'info_v>,
    #[account(
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_v, Market>,
    #[account(
        seeds = [b"escrow", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Account<'info_v, Escrow>,
    #[account(
        init_if_needed,
        space = Poll::INIT_SPACE,
        payer = signer,
        seeds = [b"poll", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Account<'info_v, Poll>,
    #[account(
        init_if_needed,
        space = Voter::INIT_SPACE,
        payer = signer,
        seeds = [b"voter", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), params.address.as_ref()],
        bump,
    )]
    pub voter: Account<'info_v, Voter>,
    pub system_program: Program<'info_v, System>,
}

impl<'info_v> Vote<'info_v> {
    pub fn add_vote(
        &mut self,
        bumps: &VoteBumps,
        params: &MarketParams,
        amount: u64,
        direction: bool,
    ) -> Result<()> {

        let time: i64 = Clock::get()?.unix_timestamp;

        // // This condition will also need to change if we decide to do time-based poll closing
        // if self.poll.total_for + self.poll.total_against < VOTE_THRESHOLD {
        //     self.market.set_inner(
        //         Market {
        //             bump: self.market.bump,             // u8
        //             token: self.market.token,           // Pubkey
        //             facets: self.market.facets.clone(), // Vec<Facet>
        //             state: MarketState::Inactive,       // MarketState
        //             round: self.market.round,           // u16
        //         }
        //     );

        //     return Err(anchor_lang::error!(VotingError::VotingClosed));
        // }

        require!(self.escrow.end_time < time, VotingError::NotVotingTime);
        require!(!self.poll.voters.as_ref().unwrap().contains(&params.address), VotingError::AlreadyVoted);
        require!(self.poll.total_for + self.poll.total_against < VOTE_THRESHOLD, VotingError::VotingClosed);   // Better to do time- or threshold-based?

        if self.market.state != MarketState::Voting && self.escrow.end_time < time {
            self.market.set_inner(
                Market {
                    bump: self.market.bump,             // u8
                    token: self.market.token,           // Pubkey
                    facets: self.market.facets.clone(), // Vec<Facet>
                    state: MarketState::Voting,         // MarketState
                    round: self.market.round,           // u16
                }
            );
        }

        require!(self.market.state == MarketState::Voting, VotingError::NotVotingTime);

        // Receive voting tokens from signer
        self.receive_vote_token(self.signer.to_account_info(), amount);

        // Update poll + voter
        let total_for: u64 = match direction {
            true => amount,
            false => 0_u64
        };
        
        let total_against: u64 = amount - total_for;

        if self.poll.total_for + self.poll.total_against == 0 {
            self.poll.set_inner(
                Poll {
                    bump: bumps.poll,                           // u8
                    market: params.authensus_token,             // Pubkey
                    facet: self.escrow.facet.clone(),           // Facet
                    voters: Some(Vec::from([params.address])),  // Option<Vec<Pubkey>>
                    total_for,                                  // u64
                    total_against,                              // u64
                }
            );
        } else {
            let voters: &mut Vec<Pubkey> = &mut self.poll.voters.clone().unwrap();
            voters.push(params.address);

            self.poll.set_inner(
                Poll {
                    bump: self.poll.bump,                                   // u8
                    market: self.poll.market,                               // Pubkey
                    facet: self.poll.facet.clone(),                         // Facet
                    voters: Some(voters.clone()),                           // Option<Vec<Pubkey>>
                    total_for: self.poll.total_for + total_for,             // u64
                    total_against: self.poll.total_against + total_against, // u64
                }
            );
        }

        self.voter.set_inner(
            Voter {
                bump: bumps.voter,              // u8
                pk: params.address,             // Pubkey
                market: params.authensus_token, // Pubkey
                facet: params.facet.clone(),    // Facet
                amount,                         // u64
                direction,                      // bool
            }
        );
        
        Ok(())

    }

    fn receive_vote_token(
        &self,
        from: AccountInfo<'info_v>,
        amount: u64
    ) -> Result<()> {

        let accounts = Transfer {
            from,
            to: self.escrow.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        // THIS TRANSFERS SOL. NEEDS TO TRANSFER VOTE TOKENS
        transfer(cpi_ctx, amount)

    }

}
