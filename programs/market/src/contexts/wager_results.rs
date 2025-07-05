use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer}
};
use anchor_spl::{
    associated_token::{get_associated_token_address, AssociatedToken},
    token::{Mint, Token, TokenAccount}
};

use voting_tokens::{
    cpi::{accounts::MintTokens, mint_tokens},
    self,
    program::VotingTokens,
    id as get_voting_tokens_program_id,
};

use crate::constants::{PERCENTAGE_WINNINGS_KEPT, TREASURY_ADDRESS, VOTE_THRESHOLD};
use crate::error::{FacetError, MintError, ResultsError, TokenError, TreasuryError, VotingError};
use crate::states::{Bettor, Escrow, Market, MarketParams, MarketState, Poll};
use crate::utils::functions::compute_returns;

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct WagerResult<'info_wr> {
    #[account(mut)]
    pub treasury: Signer<'info_wr>,
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

        let mint_program_pk: Pubkey = get_voting_tokens_program_id();
        let mint_pk: Pubkey= Pubkey::find_program_address(
            &[b"mint"],
            &mint_program_pk,
        ).0;

        let signer_ata: Pubkey = get_associated_token_address(
             &self.signer.key(),
             &mint_pk,
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
        //  - The person should not yet have had their votes consolidated       |       √
        //  - Market should contain the given facet                             |       √
        //  - The token must be the same as that which instantiated the market  |       √
        //  - Treasury should have the expected address                         |       √
        //  - ATA needs to be correct                                           |       √
        //  - Mint account ID needs to be correct                               |       √
        //  - Voting Tokens Program needs to be correct                         |       √
        require!(self.poll.total_for + self.poll.total_against >= VOTE_THRESHOLD, ResultsError::VotingNotFinished);
        require!(wagers_count_condition, ResultsError::NotABettor);
        require!(!consolidated_bettors_condition, ResultsError::BettorAlreadyConsolidated);
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.market.token == params.authensus_token, TokenError::NotTheSameToken);
        require!(self.treasury.key().to_string() == TREASURY_ADDRESS, TreasuryError::WrongTreasury);
        require!(signer_ata == self.recipient.key(), VotingError::IncorrectATA);
        require!(self.mint.key() == mint_pk, MintError::NotTheRightMintPK);
        require!(self.voting_tokens_program.key() == get_voting_tokens_program_id(), MintError::NotTheRightMintProgramPK);

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

        // Reset bettor account numbers
        self.bettor.tot_for = 0_u64;
        self.bettor.tot_against = 0_u64;
        self.bettor.tot_underdog = 0_u64;

        if bet_returned == 0 {
            return Ok(())
        }

        // Authensus keeps a share of winnings
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

        // Reset bettor account numbers
        self.bettor.tot_for = 0_u64;
        self.bettor.tot_against = 0_u64;
        self.bettor.tot_underdog = 0_u64;
        
        self.reimburse_sol_wager(total_bets)

    }

    fn reimburse_sol_wager(
        &self,
        amount: u64,
    ) -> Result<()> {

        let accounts = Transfer {
            from: self.treasury.to_account_info(),
            to: self.signer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, amount)

    }

    fn mint_voting_tokens_to_winner(
        &mut self,
        winnings: u64,
    ) -> Result<()> {

        let accounts: MintTokens<'_> = MintTokens{
            payer: self.treasury.to_account_info(),
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
