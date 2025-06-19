use anchor_lang::{
    prelude::*,
    system_program::{Transfer, transfer}
};

use crate::constants::{DIV_BUFFER, PERCENTAGE_WINNINGS_KEPT, VOTE_THRESHOLD};
use crate::error::ResultsError;
use crate::states::{Bettor, Escrow, Facet, Market, MarketState, Poll};

#[derive(Accounts)]
#[instruction(authensus_token: Pubkey, facet: Facet, address: Pubkey)]
pub struct WagerResult<'info_wr> {
    #[account(mut)]
    pub signer: Signer<'info_wr>,
    #[account(
        seeds = [b"market", authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_wr, Market>,
    #[account(
        seeds = [b"escrow", authensus_token.as_ref(), facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Account<'info_wr, Escrow>,
    #[account(
        seeds = [b"bettor", authensus_token.as_ref(), facet.to_string().as_bytes(), address.as_ref()],
        bump,
    )]
    pub bettor: Account<'info_wr, Bettor>,
    #[account(
        seeds = [b"poll", authensus_token.as_ref(), facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Account<'info_wr, Poll>,
    pub system_program: Program<'info_wr, System>,
}

impl<'info_wr> WagerResult<'info_wr> {

    pub fn distribute_tokens_to_bettors_and_assign_markets(
        &mut self,
        _address: Pubkey,
        _authensus_token: Pubkey,
        _facet: Facet,
    ) -> Result<()> {

        require!(self.poll.total_for + self.poll.total_against >= VOTE_THRESHOLD, ResultsError::VotingNotFinished);
        require!(self.escrow.bettors.as_ref().unwrap().contains(&self.bettor.pk), ResultsError::NotABettor);
        // require!(self.escrow.total_underdog == 0, ResultsError::UnderdogBetsNotResolved);

        // Change the market state if necessary
        if self.market.state != MarketState::Inactive {
            self.market.set_inner(
                Market {
                    bump: self.market.bump,             // u8
                    token: self.market.token,           // Pubkey
                    facets: self.market.facets.clone(), // Vec<Facet>
                    state: MarketState::Inactive,       // MarketState
                    round: self.market.round,           // u16
                }
            );
        }

        if self.poll.total_for == self.poll.total_against {
            return voting_tie();
        }

        let direction = self.poll.total_for > self.poll.total_against;

        let (bet_returned, winnings_pre) = compute_returns(
            direction,
            self.escrow.tot_for,
            self.escrow.tot_against,
            self.escrow.tot_underdog,
            self.bettor.tot_for,
            self.bettor.tot_against,
            self.bettor.tot_underdog,
        );

        let winnings: u64 = (PERCENTAGE_WINNINGS_KEPT*winnings_pre)/100;

        // Reimburse bets
        reimburse_sol_wager(self.signer.to_account_info(), bet_returned);

        // Mint and allocate voting tokens
        assign_markets_to_new_voter()

    }

    fn assign_markets_to_new_voter(&mut self) -> Result<()> {

        require!(self.market.state == MarketState::Inactive, ResultsError::VotingNotFinished);

        Ok(())

    }

    fn voting_tie(
        &mut self
    ) -> Result<()> {

        let total_bets = self.bettor.tot_for + self.bettor.tot_against + self.bettor.tot_underdog;
        reimburse_sol_wager(self.signer.to_account_info(), total_bets)

    }

    fn compute_returns(
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

    fn reimburse_sol_wager(&self, to: AccountInfo<'info_wr>, amount: u64) -> Result<()> {

        let accounts = Transfer {
            from: self.escrow.to_account_info(),
            to,
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, amount)

    }

}
