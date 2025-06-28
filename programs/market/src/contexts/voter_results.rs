use anchor_lang::prelude::*;

use treasury::{
    cpi::{accounts::Transact, reimburse},
    program::TreasuryProgram,
    self,
    Treasury,
};

use crate::error::ResultsError;
use crate::states::{Market, MarketParams, MarketState, Poll, Voter};

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct VoterResult<'info_vr> {
    #[account(mut)]
    pub signer: Signer<'info_vr>,
    #[account(
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_vr, Market>,
    #[account(
        seeds = [b"poll", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Account<'info_vr, Poll>,
    #[account(
        seeds = [b"voter", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), params.address.as_ref()],
        bump,
    )]
    pub voter: Account<'info_vr, Voter>,
    #[account(mut)]
    pub treasury: Account<'info_vr, Treasury>,
    pub treasury_program: Program<'info_vr, TreasuryProgram>,
    pub system_program: Program<'info_vr, System>,
}

/*
pub struct Transact<'info_t> {
    #[account(mut)]
    pub signer: Signer<'info_t>,
    #[account(mut)]
    pub coparty: Signer<'info_t>,
    #[account(
        mut,
        seeds = [b"treasury"],
        bump,
    )]
    pub treasury: Account<'info_t, Treasury>,
    pub system_program: Program<'info_t, System>,
}
*/

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

        if winnings == 0 {
            return Ok(())
        }

        let accounts = Transact {
            signer: self.signer.to_account_info(),
            coparty: self.voter.to_account_info(),
            treasury: self.treasury.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            self.treasury_program.to_account_info(),
            accounts,
        );

        reimburse(
            cpi_ctx,
            winnings,
        )

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
