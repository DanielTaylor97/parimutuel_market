use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer}
};

use crate::states::{Bettor, Escrow, Market, MarketParams, MarketState};
use crate::error::*;

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct Wager<'info_w> {
    #[account(mut)]
    pub signer: Signer<'info_w>,
    #[account(
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_w, Market>,
    #[account(
        seeds = [b"escrow", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Account<'info_w, Escrow>,
    #[account(
        init_if_needed,
        space = Bettor::INIT_SPACE,
        payer = signer,
        seeds = [b"bettor", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), params.address.as_ref()],
        bump,
    )]
    pub bettor: Account<'info_w, Bettor>,
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

        if self.escrow.end_time < time {

            self.market.set_inner(
                Market {
                    bump: self.market.bump,             // u8
                    token: self.market.token,           // Pubkey
                    facets: self.market.facets.clone(), // Vec<Facet>
                    state: MarketState::Voting,         // MarketState
                    round: self.market.round,           // u16
                }
            );

            return Ok(())
        }

        require!(self.market.state == MarketState::Betting, BettingError::MarketNotInBettingState);
        require!(self.escrow.end_time >= time, BettingError::MarketNotInBettingState);
        require!(self.bettor.get_lamports() > amount, BettingError::InsufficientFunds);
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.bettor.tot_underdog == 0, BettingError::BetWithUnderdogBet);

        self.receive_sol_wager(self.signer.to_account_info(), amount);

        let amount_for: u64 = match direction {
            true => amount,
            false => 0_u64
        };
        
        let amount_against: u64 = amount - amount_for;

        if self.bettor.tot_against == 0 && self.bettor.tot_against == 0 {
            self.bettor.set_inner(
                Bettor {
                    bump: bumps.bettor,             // u8
                    pk: params.address,             // Pubkey
                    market: params.authensus_token, // Pubkey
                    facet: params.facet.clone(),    // Facet
                    tot_for: amount_for,            // u64
                    tot_against: amount_against,    // u64
                    tot_underdog: 0_u64             // u64
                }
            );
        } else {
            self.bettor.set_inner(
                Bettor {
                    bump: bumps.bettor,                                     // u8
                    pk: self.bettor.pk,                                     // Pubkey
                    market: self.bettor.market,                             // Pubkey
                    facet: self.bettor.facet.clone(),                       // Facet
                    tot_for: self.bettor.tot_for + amount_for,              // u64
                    tot_against: self.bettor.tot_against + amount_against,  // u64
                    tot_underdog: self.bettor.tot_underdog,                 // u64
                }
            );
        }

        let bettors_clone = &mut self.escrow.bettors.clone().unwrap();

        if !bettors_clone.contains(&params.address) {
            bettors_clone.push(params.address);
        }

        self.escrow.set_inner(
            Escrow {
                bump: self.escrow.bump,                                 // u8
                initialiser: self.escrow.initialiser,                   // Pubkey
                market: self.escrow.market,                             // Pubkey
                facet: self.escrow.facet.clone(),                       // Facet
                bettors: Some(bettors_clone.clone()),                   // Option<Vec<Pubkey>>
                start_time: self.escrow.start_time,                     // i64
                end_time: self.escrow.end_time,                         // i64
                tot_for: self.escrow.tot_for + amount_for,              // u64
                tot_against: self.escrow.tot_against + amount_against,  // u64
                tot_underdog: self.escrow.tot_underdog,                 // u64
            }
        );
        
        Ok(())

    }

    pub fn underdog_bet(
        &mut self,
        bumps: &WagerBumps,
        params: &MarketParams,
        amount: u64,
    ) -> Result<()> {

        let time: i64 = Clock::get()?.unix_timestamp;

        if self.escrow.end_time < time {

            self.market.set_inner(
                Market {
                    bump: self.market.bump,             // u8
                    token: self.market.token,           // Pubkey
                    facets: self.market.facets.clone(), // Vec<Facet>
                    state: MarketState::Voting,         // MarketState
                    round: self.market.round,           // u16
                }
            );

            return Ok(())
        }

        require!(self.market.state == MarketState::Betting, BettingError::MarketNotInBettingState);
        require!(self.escrow.end_time >= time, BettingError::MarketNotInBettingState);
        require!(self.bettor.get_lamports() > amount, BettingError::InsufficientFunds);
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.escrow.tot_for + self.escrow.tot_against > 0, BettingError::UnderdogBetTooEarly);
        require!(self.bettor.tot_for > 0 || self.bettor.tot_against > 0, BettingError::UnderdogWithOtherBet);

        self.receive_sol_wager(self.signer.to_account_info(), amount);

        if self.bettor.tot_underdog == 0 {
            self.bettor.set_inner(
                Bettor {
                    bump: bumps.bettor,         // u8
                    pk: params.address,                // Pubkey
                    market: params.authensus_token,    // Pubkey
                    facet: params.facet.clone(),                      // Facet
                    tot_for: 0_u64,             // u64
                    tot_against: 0_u64,         // u64
                    tot_underdog: amount,       // u64
                }
            );
        } else {
            self.bettor.set_inner(
                Bettor {
                    bump: bumps.bettor,                                 // u8
                    pk: self.bettor.pk,                                 // Pubkey
                    market: self.bettor.market,                         // Pubkey
                    facet: self.bettor.facet.clone(),                   // Facet
                    tot_for: self.bettor.tot_for,                       // u64
                    tot_against: self.bettor.tot_against,               // u64
                    tot_underdog: self.bettor.tot_underdog + amount,    // u64
                }
            );
        }

        let bettors_clone = &mut self.escrow.bettors.clone().unwrap();

        if !bettors_clone.contains(&params.address) {

            bettors_clone.push(params.address);

            self.escrow.set_inner(
                Escrow {
                    bump: self.escrow.bump,                             // u8
                    initialiser: self.escrow.initialiser,               // Pubkey
                    market: self.escrow.market,                         // Pubkey
                    facet: self.escrow.facet.clone(),                   // Facet
                    bettors: Some(bettors_clone.clone()),               // Option<Vec<Pubkey>>
                    start_time: self.escrow.start_time,                 // i64
                    end_time: self.escrow.end_time,                     // i64
                    tot_for: self.escrow.tot_for,                       // u64
                    tot_against: self.escrow.tot_against,               // u64
                    tot_underdog: self.escrow.tot_underdog + amount,    // u64
                }
            );
        }
        
        Ok(())

    }

    fn receive_sol_wager(&self, from: AccountInfo<'info_w>, amount: u64) -> Result<()> {

        let accounts = Transfer {
            from,
            to: self.escrow.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, amount)

    }

}
