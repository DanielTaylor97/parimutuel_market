use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

use treasury::{
    self,
    Treasury,
};

use crate::states::{Bettor, Escrow, Market, MarketParams, MarketState};
use crate::constants::TREASURY_AUTHORITY;
use crate::error::{BettingError, FacetError, TokenError, TreasuryError};

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct StartMarket<'info_s> {
    #[account(mut)]
    pub signer: Signer<'info_s>,
    #[account(mut)]
    pub treasury_auth: Signer<'info_s>,
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
        seeds = [b"bettor", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), signer.key().as_ref()],
        bump,
    )]
    pub initialiser: Account<'info_s, Bettor>,
    #[account(mut)]
    pub treasury: Account<'info_s, Treasury>,       // Should already be initialised
    pub system_program: Program<'info_s, System>,
}

impl<'info_s> StartMarket<'info_s> {

    pub fn start(
        &mut self,
        bumps: &StartMarketBumps,
        params: &MarketParams,
    ) -> Result<()> {

        // Requirements:                                                        |   Implemented:
        //  - The given facet must exist in the market                          |       √
        //  - The token must be the same as that which instantiated the market  |       √
        //  - There should be no bottors and no bets in the escrow              |       √
        //  - Treasury authority should be the same as treasury_auth            |       √
        //  - Treasury authority should be the same as on record                |       √
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.market.token == params.authensus_token, TokenError::NotTheSameToken);
        require!(self.escrow.bettors == None && self.escrow.tot_for + self.escrow.tot_against == 0, BettingError::StartingWithBetsInPlace);
        require!(self.treasury_auth.key() == self.treasury.authority, TreasuryError::TreasuryAuthoritiesDontMatch);
        require!(self.treasury_auth.key().to_string() == TREASURY_AUTHORITY, TreasuryError::WrongTreasuryAuthority);

        let start_time = Clock::get()?.unix_timestamp;

        self.escrow.set_inner(
            Escrow {
                bump: bumps.escrow,             // u8
                initialiser: self.signer.key(), // Pubkey
                market: params.authensus_token, // Pubkey
                facet: params.facet.clone(),    // Facet
                bettors: None,                  // Option<Vec<Pubkey>>
                bettors_consolidated: None,     // Option<Vec<Pubkey>>
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
                start_time,                         // i64
                timeout: self.market.timeout,       // i64
                state: MarketState::Betting,        // MarketState
                round: self.market.round + 1,       // u16
            }
        );

        Ok(())
        
    }

    pub fn first_bet(
        &mut self,
        bumps: &StartMarketBumps,
        params: &MarketParams,
        amount: u64,
        direction: bool,
    ) -> Result<()> {

        // Requirements:                                                        |   Implemented:
        //  - The given facet must exist in the market                          |       √
        //  - The token must be the same as that which instantiated the market  |       √
        //  - There should be no bottors and no bets in the escrow              |       √
        //  - Initialiser should have sufficient funds to make the bet          |       √
        //  - Market should now be in a betting state                           |       √
        //  - Treasury authority should be the same as treasury_auth            |       √
        //  - Treasury authority should be the same as on record                |       √
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.market.token == params.authensus_token, TokenError::NotTheSameToken);
        require!(self.escrow.bettors == None && self.escrow.tot_for + self.escrow.tot_against == 0, BettingError::StartingWithBetsInPlace);
        require!(self.initialiser.get_lamports() > amount, BettingError::InsufficientFunds);
        require!(self.market.state == MarketState::Betting, BettingError::MarketNotInBettingState);
        require!(self.treasury_auth.key() == self.treasury.authority, TreasuryError::TreasuryAuthoritiesDontMatch);
        require!(self.treasury_auth.key().to_string() == TREASURY_AUTHORITY, TreasuryError::WrongTreasuryAuthority);

        self.receive_sol_start(self.signer.to_account_info(), amount)?;

        let tot_for: u64 = match direction {
            true => amount,
            false => 0_u64
        };
        
        let tot_against = amount - tot_for;

        self.escrow.set_inner(
            Escrow {
                bump: self.escrow.bump,                         // u8
                initialiser: self.escrow.initialiser,           // Pubkey
                market: self.escrow.market,                     // Pubkey
                facet: self.escrow.facet.clone(),               // Facet
                bettors: Some(Vec::from([self.signer.key()])),  // Option<Vec<Pubkey>>
                bettors_consolidated: None,                     // Option<Vec<Pubkey>>
                tot_for,                                        // u64
                tot_against,                                    // u64
                tot_underdog: self.escrow.tot_underdog,         // u64
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
            to: self.treasury_auth.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, amount)

    }

}
