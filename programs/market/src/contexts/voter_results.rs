use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount}
};

use treasury::{
    cpi::{accounts::Transact, reimburse},
    program::TreasuryProgram,
    self,
    Treasury,
};
use voting_tokens::{
    cpi::{accounts::MintTokens, mint_tokens},
    self,
    program::VotingTokens,
};

use crate::constants::VOTING_TOKENS_PROGRAM_ID;
use crate::error::{CpiError, ResultsError};
use crate::states::{Market, MarketParams, MarketState, Poll, Voter};

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct VoterResult<'info_vr> {
    #[account(mut)]
    pub treasury_auth: Signer<'info_vr>,
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
    pub voting_token_account: Account<'info_vr, TokenAccount>,
    #[account(mut)]
    pub treasury: Account<'info_vr, Treasury>,
    pub treasury_program: Program<'info_vr, TreasuryProgram>,
    pub associated_token_program: Program<'info_vr, AssociatedToken>,
    #[account(mut)]
    pub mint: Account<'info_vr, Mint>,
    pub system_program: Program<'info_vr, System>,
    pub token_program: Program<'info_vr, Token>,
    pub rent: Sysvar<'info_vr, Rent>,
    pub voting_tokens_program: Program<'info_vr, VotingTokens>,
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

        if winnings == 0 {
            return Ok(())
        }

        let accounts = Transact {
            signer: self.treasury_auth.to_account_info(),                               // This needs to be the treasury authority
            coparty: self.signer.to_account_info(),                                     // This needs to be the person receiving the reimbursement
            treasury: self.treasury.to_account_info(),
            voting_token_account: self.voting_token_account.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
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

        // In the case of a tie everyone gets their votes tokens re-minted
        self.reimburse_votes(self.voting_token_account.to_account_info(), self.voter.amount)

    }

    fn reimburse_votes(
        &self,
        to: AccountInfo<'info_vr>,
        amount: u64
    ) -> Result<()> {

        require!(self.market.state == MarketState::Consolidating, ResultsError::VotingNotFinished);

        let program_account: AccountInfo<'_> = self.voting_tokens_program.to_account_info();

        require!(program_account.key().to_string() == VOTING_TOKENS_PROGRAM_ID, CpiError::WrongProgramID);

        let accounts: MintTokens<'_> = MintTokens{
            payer: self.signer.to_account_info(),
            mint: self.mint.to_account_info(),
            recipient: to,
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            rent: self.rent.to_account_info(),
        };

        let cpi_ctx: CpiContext<'_, '_, '_, '_, MintTokens<'_>> = CpiContext::new(
            program_account,
            accounts,
        );

        mint_tokens(
            cpi_ctx,
            amount,
        )?;

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
