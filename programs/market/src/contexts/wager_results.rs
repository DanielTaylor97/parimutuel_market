use anchor_lang::{
    prelude::*,
    system_program::{Transfer, transfer}
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount}
};

use voting_tokens::{
    cpi::{accounts::MintTokens, mint_tokens},
    self,
    program::VotingTokens,
};

use crate::constants::{DIV_BUFFER, PERCENTAGE_WINNINGS_KEPT, VOTE_THRESHOLD, VOTING_TOKENS_PROGRAM_ID};
use crate::error::{CpiError, ResultsError};
use crate::states::{Bettor, Escrow, Market, MarketParams, MarketState, Poll};

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct WagerResult<'info_wr> {
    #[account(mut)]
    pub signer: Signer<'info_wr>,
    #[account(
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_wr, Market>,
    #[account(
        seeds = [b"escrow", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Account<'info_wr, Escrow>,
    #[account(
        seeds = [b"bettor", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), params.address.as_ref()],
        bump,
    )]
    pub bettor: Account<'info_wr, Bettor>,
    #[account(
        seeds = [b"poll", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Account<'info_wr, Poll>,
    #[account(mut)]
    pub mint: Account<'info_wr, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer,
    )]
    pub recipient: Account<'info_wr, TokenAccount>,
    pub system_program: Program<'info_wr, System>,
    pub token_program: Program<'info_wr, Token>,
    pub associated_token_program: Program<'info_wr, AssociatedToken>,
    pub rent: Sysvar<'info_wr, Rent>,
    pub voting_tokens_program: Program<'info_wr, VotingTokens>,
}

impl<'info_wr> WagerResult<'info_wr> {

    pub fn distribute_tokens_to_bettors_and_assign_markets(
        &mut self,
        params: &MarketParams,
    ) -> Result<()> {

        // Requirements:
        //  - Voting is finished
        //  - Given address is a bettor
        //  - Signer is the bettor
        require!(self.poll.total_for + self.poll.total_against >= VOTE_THRESHOLD, ResultsError::VotingNotFinished);
        require!(self.escrow.bettors.as_ref().unwrap().contains(&params.address), ResultsError::NotABettor);
        require!(self.signer.key == &params.address, ResultsError::SignerNotPK);
        // require!(self.escrow.total_underdog == 0, ResultsError::UnderdogBetsNotResolved);

        // Change the market state if necessary
        if self.market.state != MarketState::Consolidating {
            self.market.set_inner(
                Market {
                    bump: self.market.bump,             // u8
                    token: self.market.token,           // Pubkey
                    facets: self.market.facets.clone(), // Vec<Facet>
                    state: MarketState::Consolidating,       // MarketState
                    round: self.market.round,           // u16
                }
            );
        }

        if self.poll.total_for == self.poll.total_against {
            return self.voting_tie();
        }

        let direction = self.poll.total_for > self.poll.total_against;

        let (bet_returned, winnings_pre) = self.compute_returns(
            direction,
            self.escrow.tot_for,
            self.escrow.tot_against,
            self.escrow.tot_underdog,
            self.bettor.tot_for,
            self.bettor.tot_against,
            self.bettor.tot_underdog,
        );

        if bet_returned == 0 {
            return Ok(())
        }

        let winnings: u64 = (PERCENTAGE_WINNINGS_KEPT*winnings_pre)/100;

        // Reimburse bets
        self.reimburse_sol_wager(self.signer.to_account_info(), bet_returned)?;

        // Mint and allocate voting tokens
        self.assign_markets_to_new_voter(winnings)

    }

    fn assign_markets_to_new_voter(
        &mut self,
        winnings: u64,
    ) -> Result<()> {

        require!(self.market.state == MarketState::Consolidating, ResultsError::VotingNotFinished);

        let program_account: AccountInfo<'_> = self.voting_tokens_program.to_account_info();

        require!(program_account.key().to_string() == VOTING_TOKENS_PROGRAM_ID, CpiError::WrongProgramID);

        let accounts: MintTokens<'_> = MintTokens{
            payer: self.signer.to_account_info(),
            mint: self.mint.to_account_info(),
            recipient: self.recipient.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            rent: self.rent.to_account_info(),
        };

        // No PDAs required for the CPI, so we use new() and not new_with_signer()
        let cpi_ctx: CpiContext<'_, '_, '_, '_, MintTokens<'_>> = CpiContext::new(
            program_account,
            accounts,
        );

        mint_tokens(
            cpi_ctx,
            winnings,
        )?;

        // TODO: ACTUALLY ASSIGN NEW MARKETS

        Ok(())

    }

    fn voting_tie(
        &mut self
    ) -> Result<()> {

        let total_bets = self.bettor.tot_for + self.bettor.tot_against + self.bettor.tot_underdog;
        self.reimburse_sol_wager(self.signer.to_account_info(), total_bets)

    }

    fn compute_returns(
        &mut self,
        direction: bool,
        escrow_tot_for: u64,
        escrow_tot_against: u64,
        escrow_tot_underdog: u64,
        bettor_tot_for: u64,
        bettor_tot_against: u64,
        bettor_tot_underdog: u64,
    ) -> (u64, u64) {
        let for_multiplier: u64 = match direction {
            true => 1_u64,
            false => 0_u64,
        };
        let against_multiplier: u64 = 1_u64 - for_multiplier;

        // Escrow totals for and against with appropriate shares from underdog bets
        let final_tot_for: u64 = (DIV_BUFFER*escrow_tot_for + (DIV_BUFFER*escrow_tot_underdog*escrow_tot_against)/(escrow_tot_for + escrow_tot_against))/DIV_BUFFER;
        let final_tot_against: u64 = (DIV_BUFFER*escrow_tot_against + (DIV_BUFFER*escrow_tot_underdog*escrow_tot_for)/(escrow_tot_for + escrow_tot_against))/DIV_BUFFER;

        // Final bets for and against from user underdog bets
        let underdog_for: u64 = ((DIV_BUFFER*bettor_tot_underdog*final_tot_against)/(final_tot_against + final_tot_against))/DIV_BUFFER;
        let underdog_against: u64 = ((DIV_BUFFER*bettor_tot_underdog*final_tot_for)/(final_tot_against + final_tot_against))/DIV_BUFFER;

        // Winnings for and against from user normal bets
        let winnings_for: u64 = ((DIV_BUFFER*final_tot_against*bettor_tot_for)/final_tot_for)/DIV_BUFFER;
        let winnings_against: u64 = ((DIV_BUFFER*final_tot_for*bettor_tot_against)/final_tot_against)/DIV_BUFFER;

        // Winnings for and against from user underdog bets
        let underdog_winnings_for: u64 = ((DIV_BUFFER*final_tot_against*underdog_for)/final_tot_for)/DIV_BUFFER;
        let underdog_winnings_against: u64 = ((DIV_BUFFER*final_tot_for*underdog_against)/final_tot_against)/DIV_BUFFER;

        let bet_returned: u64 = for_multiplier*(bettor_tot_for + underdog_for)
                                + against_multiplier*(bettor_tot_against + underdog_against);
        let winnings_pre: u64 = for_multiplier*(winnings_for + underdog_winnings_for)
                                + against_multiplier*(winnings_against + underdog_winnings_against);

        return (bet_returned, winnings_pre)

    }

    fn reimburse_sol_wager(
        &self,
        to: AccountInfo<'info_wr>,
        amount: u64
    ) -> Result<()> {

        let accounts = Transfer {
            from: self.escrow.to_account_info(),
            to,
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, amount)

    }

}
