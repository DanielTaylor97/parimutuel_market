use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer}
};

use treasury::{
    self,
    Treasury,
};

use crate::states::{Bettor, Escrow, Market, MarketParams, MarketState};
use crate::constants::{MAX_WAGERS, TREASURY_AUTHORITY};
use crate::error::{BettingError, FacetError, TokenError, TreasuryError};

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct Wager<'info_w> {
    #[account(mut)]
    pub treasury_auth: Signer<'info_w>,
    #[account(mut)]
    pub signer: Signer<'info_w>,
    #[account(
        mut,
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_w, Market>,
    #[account(
        mut,
        seeds = [b"escrow", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Account<'info_w, Escrow>,
    #[account(
        init_if_needed,
        space = Bettor::INIT_SPACE,
        payer = signer,
        seeds = [b"bettor", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), signer.key().as_ref()],
        bump,
    )]
    pub bettor: Account<'info_w, Bettor>,
    #[account(mut)]
    pub treasury: Account<'info_w, Treasury>,       // Should already be initialised
    pub system_program: Program<'info_w, System>,
}

impl<'info_w> Wager<'info_w> {

    pub fn place_wager(
        &mut self,
        bumps: &WagerBumps,
        params: &MarketParams,
        amount: u64,
        direction: bool,
    ) -> Result<()> {

        let time: i64 = Clock::get()?.unix_timestamp;

        let wagers_count_condition: bool = match self.escrow.bettors.is_some() {
            true => self.escrow.bettors.as_ref().unwrap().len() < MAX_WAGERS.into(),
            false => true,
        };

        // Requirements:                                                        |   Implemented:
        //  - Market should be in a betting state                               |       √
        //  - Bettor should have sufficient balance to place the bet            |       √
        //  - Market should contain the given facet                             |       √
        //  - The token must be the same as that which instantiated the market  |       √
        //  - Bettor should not have placed any underdog bets                   |       √
        //  - Treasury authority should be the same as treasury_auth            |       √
        //  - Treasury authority should be the same as on record                |       √
        //  - Current number of wagers must be less than the max                |       √
        require!(self.market.state == MarketState::Betting, BettingError::MarketNotInBettingState);
        require!(self.bettor.get_lamports() > amount, BettingError::InsufficientFunds);
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.market.token == params.authensus_token, TokenError::NotTheSameToken);
        require!(self.bettor.tot_underdog == 0, BettingError::BetWithUnderdogBet);
        require!(self.treasury_auth.key() == self.treasury.authority, TreasuryError::TreasuryAuthoritiesDontMatch);
        require!(self.treasury_auth.key().to_string() == TREASURY_AUTHORITY, TreasuryError::WrongTreasuryAuthority);
        require!(wagers_count_condition, BettingError::TooManyBettors);

        // If the market has timed out then abort the bet after setting the market state to MarketState::Voting
        if self.market.start_time + self.market.timeout < time {

            self.market.state = MarketState::Voting;

            return Ok(())
        }

        self.receive_sol_wager(self.signer.to_account_info(), amount)?;

        let amount_for: u64 = match direction {
            true => amount,
            false => 0_u64
        };
        
        let amount_against: u64 = amount - amount_for;

        if self.bettor.tot_against == 0 && self.bettor.tot_against == 0 {
            self.bettor.set_inner(
                Bettor {
                    bump: bumps.bettor,             // u8
                    pk: self.signer.key(),          // Pubkey
                    market: params.authensus_token, // Pubkey
                    facet: params.facet.clone(),    // Facet
                    tot_for: amount_for,            // u64
                    tot_against: amount_against,    // u64
                    tot_underdog: 0_u64             // u64
                }
            );
        } else {
            self.bettor.tot_for += amount_for;
            self.bettor.tot_against += amount_against;
        }

        let bettors_clone = &mut self.escrow.bettors.clone().unwrap();

        if !bettors_clone.contains(&self.signer.key()) {
            bettors_clone.push(self.signer.key());
            self.escrow.bettors = Some(bettors_clone.clone());
        }

        self.escrow.tot_for += amount_for;
        self.escrow.tot_against += amount_against;
        
        Ok(())

    }

    pub fn underdog_bet(
        &mut self,
        bumps: &WagerBumps,
        params: &MarketParams,
        amount: u64,
    ) -> Result<()> {

        let time: i64 = Clock::get()?.unix_timestamp;

        // Requirements:                                                                    |   Implemented:
        //  - Market should be in a betting state                                           |       √
        //  - Bettor should have sufficient balance to place the bet                        |       √
        //  - Market should contain the given facet                                         |       √
        //  - The token must be the same as that which instantiated the market              |       √
        //  - At least some normal bets have already been placed                            |       √
        //  - No other bets should have been placed by this bettor already in this market   |       √
        require!(self.market.state == MarketState::Betting, BettingError::MarketNotInBettingState);
        require!(self.bettor.get_lamports() > amount, BettingError::InsufficientFunds);
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.market.token == params.authensus_token, TokenError::NotTheSameToken);
        require!(self.escrow.tot_for + self.escrow.tot_against > 0, BettingError::UnderdogBetTooEarly);
        require!(self.bettor.tot_for + self.bettor.tot_against == 0, BettingError::UnderdogWithOtherBet);

        // If the market has timed out then abort the bet after setting the market state to MarketState::Voting
        if self.market.start_time + self.market.timeout < time {

            self.market.state = MarketState::Voting;

            return Ok(())
        }

        self.receive_sol_wager(self.signer.to_account_info(), amount)?;

        if self.bettor.tot_underdog == 0 {
            self.bettor.set_inner(
                Bettor {
                    bump: bumps.bettor,             // u8
                    pk: self.signer.key(),          // Pubkey
                    market: params.authensus_token, // Pubkey
                    facet: params.facet.clone(),    // Facet
                    tot_for: 0_u64,                 // u64
                    tot_against: 0_u64,             // u64
                    tot_underdog: amount,           // u64
                }
            );
        } else {
            self.bettor.tot_underdog += amount;
        }

        let bettors_clone = &mut self.escrow.bettors.clone().unwrap();

        if !bettors_clone.contains(&self.signer.key()) {
            bettors_clone.push(self.signer.key());
            self.escrow.bettors = Some(bettors_clone.clone());
        }

        self.escrow.tot_underdog += amount;
        
        Ok(())

    }

    fn receive_sol_wager(&self, from: AccountInfo<'info_w>, amount: u64) -> Result<()> {

        let accounts = Transfer {
            from,
            to: self.treasury_auth.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, amount)

    }

}
