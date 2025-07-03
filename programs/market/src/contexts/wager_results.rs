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

use crate::constants::{PERCENTAGE_WINNINGS_KEPT, TREASURY_AUTHORITY, TREASURY_PROGRAM_ID, VOTE_THRESHOLD, VOTING_TOKENS_MINT_ID, VOTING_TOKENS_PROGRAM_ID};
use crate::error::{CpiError, FacetError, MintError, ResultsError, TokenError, TreasuryError, VotingError};
use crate::states::{Bettor, Escrow, Market, MarketParams, MarketState, Poll};
use crate::utils::functions::compute_returns;

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct WagerResult<'info_wr> {
    #[account(mut)]
    pub treasury_auth: Signer<'info_wr>,
    #[account(mut)]
    pub signer: Signer<'info_wr>,
    #[account(
        mut,
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info_wr, Market>>,
    #[account(
        mut,
        seeds = [b"escrow", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Box<Account<'info_wr, Escrow>>,
    #[account(
        mut,
        seeds = [b"bettor", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), signer.key().as_ref()],
        bump,
    )]
    pub bettor: Box<Account<'info_wr, Bettor>>,
    #[account(
        mut,
        seeds = [b"poll", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Box<Account<'info_wr, Poll>>,
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
    pub treasury: Box<Account<'info_wr, Treasury>>,
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
            self.market.state = MarketState::Consolidating;
        }

        if self.poll.total_for == self.poll.total_against {
            return self.voting_tie();
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

    fn mint_voting_tokens_to_winner(
        &mut self,
        winnings: u64,
    ) -> Result<()> {

        require!(self.market.state == MarketState::Consolidating, ResultsError::VotingNotFinished);
        require!(self.voting_tokens_program.key().to_string() == VOTING_TOKENS_PROGRAM_ID, CpiError::WrongProgramID);

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
            self.voting_tokens_program.to_account_info(),
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

    fn add_to_consolidated(&mut self) -> Result<()> {

        if self.escrow.bettors_consolidated.is_some() {
            
            let mut consolidated_vec: Vec<Pubkey> = self.escrow.bettors_consolidated.clone().unwrap();
            consolidated_vec.push(self.signer.key());
            
            self.escrow.bettors_consolidated = Some(consolidated_vec.clone());

        } else {

            self.escrow.bettors_consolidated = Some(Vec::from([self.signer.key()]));

        }

        Ok(())
    }

}
