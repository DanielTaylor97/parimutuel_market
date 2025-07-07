use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

use crate::states::{Bettor, Escrow, Market, MarketParams, MarketState, Poll};
use crate::constants::TREASURY_ADDRESS;
use crate::error::{BettingError, FacetError, MarketError, TokenError, TreasuryError};
use crate::utils::functions::verify_signature;

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct StartMarket<'info_s> {
    #[account(mut)]
    pub signer: Signer<'info_s>,
    #[account(mut)]
    pub treasury: SystemAccount<'info_s>,
    #[account(
        mut,
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info_s, Market>>,
    #[account(
        init_if_needed,
        space = 8 + Escrow::INIT_SPACE,
        payer = signer,
        seeds = [b"escrow", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Box<Account<'info_s, Escrow>>,
    #[account(
        init_if_needed,
        space = 8 + Poll::INIT_SPACE,
        payer = signer,
        seeds = [b"poll", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Box<Account<'info_s, Poll>>,
    #[account(
        init_if_needed,
        space = 8 + Bettor::INIT_SPACE,
        payer = signer,
        seeds = [b"bettor", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), signer.key().as_ref()],
        bump,
    )]
    pub initialiser: Box<Account<'info_s, Bettor>>,
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
        //  - Market must either be in an initialised state or inactive         |       √
        //  - Treasury should have the expected address                         |       √
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.market.token == params.authensus_token, TokenError::NotTheSameToken);
        require!(self.market.state == MarketState::Initialised || self.market.state == MarketState::Inactive, MarketError::MarketInWrongState);
        require!(self.treasury.key().to_string() == TREASURY_ADDRESS, TreasuryError::WrongTreasury);

        let start_time = Clock::get()?.unix_timestamp;

        self.escrow.set_inner(
            Escrow {
                bump: bumps.escrow,         // u8
                n_bets: 0_u16,              // u16
                bets_consolidated: 0_u16,   // u16
                tot_for: 0_u64,             // u64
                tot_against: 0_u64,         // u64
                tot_underdog: 0_u64         // u64
            }
        );

        self.poll.set_inner(
            Poll {
                bump: bumps.poll,           // u8
                total_for: 0_u16,           // u16
                total_against: 0_u16,       // u16
                total_consolidated: 0_u16,  // u16
            }
        );

        self.market.start_time = start_time;
        self.market.state = MarketState::Betting;
        self.market.round += 1;

        Ok(())
        
    }

    pub fn first_bet(
        &mut self,
        bumps: &StartMarketBumps,
        params: &MarketParams,
        amount: u64,
        direction: bool,
        signed_message: [u8; 64],
    ) -> Result<()> {

        let total = (self.initialiser.tot_for + self.initialiser.tot_against + self.initialiser.tot_underdog + amount).to_string();
        let wager_message_str = params.authensus_token.to_string() + &self.market.round.to_string() + &params.facet.to_string() + &self.signer.key().to_string() + &total;
        let wager_message: &[u8] = wager_message_str.as_bytes();

        // Requirements:                                                        |   Implemented:
        //  - The given facet must exist in the market                          |       √
        //  - The token must be the same as that which instantiated the market  |       √
        //  - Provided message must be signed by the treasury account           |       √
        //  - Initialiser should have sufficient funds to make the bet          |       √
        //  - Market should now be in a betting state                           |       √
        //  - Treasury should have the expected address                         |       √
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.market.token == params.authensus_token, TokenError::NotTheSameToken);
        require!(verify_signature(signed_message, wager_message), TreasuryError::MessageNotValid);
        require!(self.initialiser.get_lamports() > amount, BettingError::InsufficientFunds);
        require!(self.market.state == MarketState::Betting, BettingError::MarketNotInBettingState);
        require!(self.treasury.key().to_string() == TREASURY_ADDRESS, TreasuryError::WrongTreasury);

        self.receive_sol_start(amount)?;

        let tot_for: u64 = match direction {
            true => amount,
            false => 0_u64
        };
        
        let tot_against = amount - tot_for;

        self.escrow.tot_for = tot_for;
        self.escrow.tot_against = tot_against;
        self.escrow.n_bets = 1;

        self.initialiser.set_inner(
            Bettor {
                bump: bumps.initialiser,            // u8
                tot_for,                            // u64
                tot_against,                        // u64
                tot_underdog: 0_u64,                // u64
                bets_signed: Some(signed_message),  // Option<[u8; 64]>
            }
        );

        Ok(())
        
    }

    fn receive_sol_start(&self, amount: u64) -> Result<()> {

        let accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.treasury.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, amount)

    }

}
