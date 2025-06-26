use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

use crate::constants::MAX_ALLOWED_TIMEOUT;
use crate::error::{BettingError, FacetError};
use crate::states::{Bettor, Escrow, Market, MarketParams, MarketState};

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct StartMarket<'info_s> {
    #[account(mut)]
    pub signer: Signer<'info_s>,
    #[account(
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_s, Market>,
    #[account(
        init_if_needed,
        space = Escrow::INIT_SPACE,
        payer = signer,
        seeds = [b"escrow", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Account<'info_s, Escrow>,
    #[account(
        init_if_needed,
        space = Bettor::INIT_SPACE,
        payer = signer,
        seeds = [b"bettor", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), params.address.as_ref()],
        bump,
    )]
    pub initialiser: Account<'info_s, Bettor>,
    pub system_program: Program<'info_s, System>,
}

impl<'info_s> StartMarket<'info_s> {

    pub fn start(
        &mut self,
        bumps: &StartMarketBumps,
        params: &MarketParams,
        timeout: i64,
    ) -> Result<()> {

        require!(timeout <= MAX_ALLOWED_TIMEOUT, BettingError::TimeoutTooLarge);
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);

        let start_time = Clock::get()?.unix_timestamp;

        self.escrow.set_inner(
            Escrow {
                bump: bumps.escrow,             // u8
                initialiser: params.address,    // Pubkey
                market: params.authensus_token, // Pubkey
                facet: params.facet.clone(),    // Facet
                bettors: None,                  // Option<Vec<Pubkey>>
                start_time,                     // i64
                end_time: start_time + timeout, // i64
                tot_for: 0_u64,                 // u64
                tot_against: 0_u64,             // u64
                tot_underdog: 0_u64             // u64
            }
        );

        self.market.set_inner(
            Market {
                bump: self.market.bump,             // u8
                token: self.market.token,           // Pubkey
                facets: self.market.facets.clone(), // Vec<Facet>
                state: MarketState::Betting,        // MarketState
                round: self.market.round + 1,       // u16
            }
        );

        Ok(())
        
    }

    pub fn first_bet(
        &mut self,
        bumps: &StartMarketBumps,
        address: Pubkey,
        amount: u64,
        direction: bool,
    ) -> Result<()> {

        require!(self.initialiser.get_lamports() > amount, BettingError::InsufficientFunds);

        self.receive_sol_start(self.signer.to_account_info(), amount)?;

        let tot_for: u64 = match direction {
            true => amount,
            false => 0_u64
        };
        
        let tot_against = amount - tot_for;

        self.escrow.set_inner(
            Escrow {
                bump: self.escrow.bump,                 // u8
                initialiser: self.escrow.initialiser,   // Pubkey
                market: self.escrow.market,             // Pubkey
                facet: self.escrow.facet.clone(),       // Facet
                bettors: Some(Vec::from([address])),    // Option<Vec<Pubkey>>
                start_time: self.escrow.start_time,     // i64
                end_time: self.escrow.end_time,         // i64
                tot_for,                                // u64
                tot_against,                            // u64
                tot_underdog: self.escrow.tot_underdog, // u64
            }
        );

        self.initialiser.set_inner(
            Bettor {
                bump: bumps.initialiser,                    // u8
                pk: self.signer.to_account_info().key(),    // Pubkey
                market: self.escrow.market,                 // Pubkey
                facet: self.escrow.facet.clone(),           // Facet
                tot_for,                                    // u64
                tot_against,                                // u64
                tot_underdog: 0_u64,                        // u64
            }
        );

        Ok(())
        
    }

    fn receive_sol_start(&self, from: AccountInfo<'info_s>, amount: u64) -> Result<()> {

        let accounts = Transfer {
            from,
            to: self.escrow.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, amount)

    }

}
