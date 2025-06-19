use anchor_lang::{
    prelude::*,
    system_program::{Transfer, transfer}
};

use crate::constants::VOTE_THRESHOLD;
use crate::error::ResultsError;
use crate::states::{Escrow, Facet, Market, MarketState, Poll, Voter};

#[derive(Accounts)]
#[instruction(authensus_token: Pubkey, facet: Facet, address: Pubkey)]
pub struct VoterResult<'info_vr> {
    #[account(mut)]
    pub signer: Signer<'info_vr>,
    #[account(
        seeds = [b"market", authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_vr, Market>,
    // #[account(
    //     seeds = [b"escrow", authensus_token.as_ref(), facet.to_string().as_bytes()],
    //     bump,
    // )]
    // pub escrow: Account<'info_vr, Escrow>,
    #[account(
        seeds = [b"poll", authensus_token.as_ref(), facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Account<'info_vr, Poll>,
    #[account(
        seeds = [b"voter", authensus_token.as_ref(), facet.to_string().as_bytes(), address.as_ref()],
        bump,
    )]
    pub voter: Account<'info_vr, Voter>,
}

impl<'info_vr> VoterResult<'info_vr> {

    pub fn distribute_sol_to_voters(
        &mut self,
        address: Pubkey,
    ) -> Result<()> {

        require!(self.market.state == MarketState::Consolidating, ResultsError::VotingNotFinished);
        require!(self.poll.voters.clone().unwrap().contains(&address), ResultsError::NotAVoter);

        if self.poll.total_for == self.poll.total_against {
            return self.voting_tie();
        }

        let direction: bool = self.poll.total_for > self.poll.total_against;

        let winnings: u64 = self.calc_winnings(direction).unwrap();

        self.payout_votes_for_sol(self.signer.to_account_info(), winnings);

        Ok(())

    }

    fn voting_tie(
        &mut self
    ) -> Result<()> {

        self.reimburse_votes(self.signer.to_account_info(), self.voter.amount)

    }

    fn reimburse_votes(
        &self,
        to: AccountInfo<'info_vr>,
        amount: u64
    ) -> Result<()> {

        // let accounts = Transfer {
        //     from: self.escrow.to_account_info(),
        //     to,
        // };

        // let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        // transfer(cpi_ctx, amount)

        Ok(())

    }

    fn payout_votes_for_sol(
        &self,
        to: AccountInfo<'info_vr>,
        amount: u64
    ) -> Result<()> {

        let accounts = Transfer {
            from: self.escrow.to_account_info(),
            to,
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, amount)

        Ok(())

    }

    // Unit test this bitch
    fn calc_winnings(
        &self,
        direction: bool,
    ) -> Result<u64> {
        match !(direction ^ self.voter.direction) {
            true => Ok(self.voter.amount),
            false => Ok(0),
        }
    }

}
