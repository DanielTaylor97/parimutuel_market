use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{get_associated_token_address_with_program_id, AssociatedToken},
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

use crate::constants::{DIV_BUFFER, PERCENTAGE_WINNINGS_KEPT, TREASURY_AUTHORITY, TREASURY_PROGRAM_ID, VOTE_THRESHOLD, VOTING_TOKENS_MINT_ID, VOTING_TOKENS_PROGRAM_ID};
use crate::error::{CpiError, FacetError, MintError, ResultsError, TokenError, TreasuryError, VotingError};
use crate::states::{Bettor, Escrow, Market, MarketParams, MarketState, Poll};

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct WagerResult<'info_wr> {
    #[account(mut)]
    pub treasury_auth: Signer<'info_wr>,
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
        seeds = [b"bettor", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), signer.key().as_ref()],
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
    #[account(mut)]
    pub treasury: Account<'info_wr, Treasury>,
    pub treasury_program: Program<'info_wr, TreasuryProgram>,
    pub voting_tokens_program: Program<'info_wr, VotingTokens>,
    pub system_program: Program<'info_wr, System>,
    pub token_program: Program<'info_wr, Token>,
    pub associated_token_program: Program<'info_wr, AssociatedToken>,
    pub rent: Sysvar<'info_wr, Rent>,
}

impl<'info_wr> WagerResult<'info_wr> {

    pub fn assign_tokens_and_markets_to_bettor(
        &mut self,
        params: &MarketParams,
    ) -> Result<()> {

        let mint_pk: Pubkey = Pubkey::from_str(VOTING_TOKENS_MINT_ID).unwrap();
        let mint_program_pk: Pubkey = Pubkey::from_str(VOTING_TOKENS_PROGRAM_ID).unwrap();

        let signer_ata: Pubkey = get_associated_token_address_with_program_id(
             &self.signer.key(),
             &mint_pk,
             &mint_program_pk,
        );

        let wagers_count_condition: bool = match self.escrow.bettors.is_some() {
            true => self.escrow.bettors.as_ref().unwrap().contains(&self.signer.key()),
            false => false,
        };

        let consolidated_bettors_condition: bool = match self.escrow.bettors_consolidated.is_some() {
            true => self.escrow.bettors_consolidated.as_ref().unwrap().contains(&self.signer.key()),
            false => false,
        };

        // Requirements:                                                        |   Implemented:
        //  - Voting is finished                                                |       √
        //  - Given address is a bettor                                         |       √
        //  - The person should not yet have had their votes consolidated
        //  - Market should contain the given facet                             |       √
        //  - The token must be the same as that which instantiated the market  |       √
        //  - Treasury authority should be the same as treasury_auth            |       √
        //  - Treasury authority should be the same as on record                |       √
        //  - ATA needs to be correct                                           |       √
        //  - Mint account ID needs to be correct                               |       √
        //  - Treasury Program needs to be correct                              |       √
        //  - Voting Tokens Program needs to be correct                         |       √
        require!(self.poll.total_for + self.poll.total_against >= VOTE_THRESHOLD.into(), ResultsError::VotingNotFinished);
        require!(wagers_count_condition, ResultsError::NotABettor);
        require!(!consolidated_bettors_condition, ResultsError::BettorAlreadyConsolidated);
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.market.token == params.authensus_token, TokenError::NotTheSameToken);
        require!(self.treasury_auth.key() == self.treasury.authority, TreasuryError::TreasuryAuthoritiesDontMatch);
        require!(self.treasury_auth.key().to_string() == TREASURY_AUTHORITY, TreasuryError::WrongTreasuryAuthority);
        require!(signer_ata == self.recipient.key(), VotingError::IncorrectATA);
        require!(self.mint.key() == mint_pk, MintError::NotTheRightMintPK);
        require!(self.treasury_program.key().to_string() == TREASURY_PROGRAM_ID, TreasuryError::NotTheRightTreasuryProgramPK);
        require!(self.voting_tokens_program.key().to_string() == VOTING_TOKENS_PROGRAM_ID, MintError::NotTheRightMintProgramPK);

        self.add_to_consolidated()?;

        // Change the market state if necessary
        if self.market.state == MarketState::Voting {
            self.market.set_inner(
                Market {
                    bump: self.market.bump,             // u8
                    token: self.market.token,           // Pubkey
                    facets: self.market.facets.clone(), // Vec<Facet>
                    start_time: self.market.start_time, // i64
                    timeout: self.market.timeout,       // i64
                    state: MarketState::Consolidating,  // MarketState
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
        self.reimburse_sol_wager(bet_returned)?;

        // Mint and allocate voting tokens
        self.mint_voting_tokens_to_winner(winnings)?;

        // Assign new markets
        self.assign_new_markets()

    }

    fn mint_voting_tokens_to_winner(
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

        Ok(())

    }

    fn assign_new_markets(&mut self) -> Result<()> {

        // TODO: ACTUALLY ASSIGN NEW MARKETS

        Ok(())
        
    }
    
    fn voting_tie(
        &mut self
    ) -> Result<()> {

        let total_bets = self.bettor.tot_for + self.bettor.tot_against + self.bettor.tot_underdog;
        self.reimburse_sol_wager(total_bets)

    }

    fn reimburse_sol_wager(
        &self,
        amount: u64,
    ) -> Result<()> {

        let cpi_accounts = Transact {
            signer: self.treasury_auth.to_account_info(),
            coparty: self.signer.to_account_info(),
            treasury: self.treasury.to_account_info(),
            voting_token_account: self.recipient.to_account_info(),
            system_program: self.system_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            self.treasury_program.to_account_info(),
            cpi_accounts,
        );

        reimburse(
            cpi_ctx,
            amount,
        )

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

    fn add_to_consolidated(&mut self) -> Result<()> {

        if self.escrow.bettors_consolidated.is_some() {
            let mut consolidated_vec: Vec<Pubkey> = self.escrow.bettors_consolidated.clone().unwrap();
            consolidated_vec.push(self.signer.key());

            self.escrow.set_inner(
                Escrow {
                    bump: self.escrow.bump,                                     // u8
                    initialiser: self.escrow.initialiser,                       // Pubkey
                    market: self.escrow.market,                                 // Pubkey
                    facet: self.escrow.facet.clone(),                           // Facet
                    bettors: self.escrow.bettors.clone(),                       // Option<Vec<Pubkey>>
                    bettors_consolidated: Some(consolidated_vec.clone()),       // Option<Vec<Pubkey>>
                    tot_for: self.escrow.tot_for,                               // u64
                    tot_against: self.escrow.tot_against,                       // u64
                    tot_underdog: self.escrow.tot_underdog,                     // u64
                }
            );
        } else {
            self.escrow.set_inner(
                Escrow {
                    bump: self.escrow.bump,                                     // u8
                    initialiser: self.escrow.initialiser,                       // Pubkey
                    market: self.escrow.market,                                 // Pubkey
                    facet: self.escrow.facet.clone(),                           // Facet
                    bettors: self.escrow.bettors.clone(),                       // Option<Vec<Pubkey>>
                    bettors_consolidated: Some(Vec::from([self.signer.key()])), // Option<Vec<Pubkey>>
                    tot_for: self.escrow.tot_for,                               // u64
                    tot_against: self.escrow.tot_against,                       // u64
                    tot_underdog: self.escrow.tot_underdog,                     // u64
                }
            );
        }

        Ok(())
    }

}
