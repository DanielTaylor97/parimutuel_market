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

use crate::constants::{TREASURY_AUTHORITY, TREASURY_PROGRAM_ID, VOTING_TOKENS_MINT_ID, VOTING_TOKENS_PROGRAM_ID};
use crate::error::{CpiError, FacetError, MintError, ResultsError, TokenError, TreasuryError, VotingError};
use crate::states::{Market, MarketParams, MarketState, Poll, Voter};
use crate::utils::functions::calc_winnings_from_votes;

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct VoterResult<'info_vr> {
    #[account(mut)]
    pub treasury_auth: Signer<'info_vr>,
    #[account(mut)]
    pub signer: Signer<'info_vr>,
    #[account(
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Box<Account<'info_vr, Market>>,
    #[account(
        mut,
        seeds = [b"poll", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Box<Account<'info_vr, Poll>>,
    #[account(
        mut,
        seeds = [b"voter", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), signer.key().as_ref()],
        bump,
    )]
    pub voter: Box<Account<'info_vr, Voter>>,
    #[account(mut)]
    pub voting_token_account: Account<'info_vr, TokenAccount>,          // Should already be initialised
    #[account(mut)]
    pub treasury_voting_token_account: Account<'info_vr, TokenAccount>, // This should already be initialised with the treasury
    #[account(mut)]
    pub treasury: Box<Account<'info_vr, Treasury>>,
    pub treasury_program: Program<'info_vr, TreasuryProgram>,
    pub associated_token_program: Program<'info_vr, AssociatedToken>,
    #[account(mut)]
    pub mint: Account<'info_vr, Mint>,
    pub system_program: Program<'info_vr, System>,
    pub token_program: Program<'info_vr, Token>,
    pub rent: Sysvar<'info_vr, Rent>,
    pub voting_tokens_program: Program<'info_vr, VotingTokens>,
}

impl<'info_vr> VoterResult<'info_vr> {

    pub fn distribute_sol_to_voter(
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
        let treasury_authority_ata: Pubkey = get_associated_token_address_with_program_id(
            &self.treasury.authority,
            &mint_pk,
            &mint_program_pk,
        );

        let voters_count_condition: bool = match self.poll.voters.is_some() {
            true => self.poll.voters.as_ref().unwrap().contains(&self.signer.key()),
            false => false,
        };

        let consolidated_voters_condition: bool = match self.poll.voters_consolidated.is_some() {
            true => self.poll.voters_consolidated.as_ref().unwrap().contains(&self.signer.key()),
            false => false,
        };

        // Requirements:                                                                                        |   Implemented:
        //  - Market should now be in the consolidation state (i.e. should only be called after wager results)  |       √
        //  - The person should be a voter in the poll                                                          |       √
        //  - The person should not yet have had their votes consolidated                                       |       √
        //  - Market should contain the given facet                                                             |       √
        //  - The token must be the same as that which instantiated the market                                  |       √
        //  - Treasury authority should be the same as treasury_auth                                            |       √
        //  - Treasury authority should be the same as on record                                                |       √
        //  - ATA needs to be correct                                                                           |       √
        //  - Mint PK needs to be correct                                                                       |       √
        //  - Treasury Program needs to be correct                                                              |       √
        //  - Voting Tokens Program needs to be correct                                                         |       √
        //  - treasury_voting_token_account should be derivable from treasury authority                         |       √
        require!(self.market.state == MarketState::Consolidating, ResultsError::VotingNotFinished);
        require!(voters_count_condition, ResultsError::NotAVoter);
        require!(!consolidated_voters_condition, ResultsError::VoterAlreadyConsolidated);
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.market.token == params.authensus_token, TokenError::NotTheSameToken);
        require!(self.treasury_auth.key() == self.treasury.authority, TreasuryError::TreasuryAuthoritiesDontMatch);
        require!(self.treasury_auth.key().to_string() == TREASURY_AUTHORITY, TreasuryError::WrongTreasuryAuthority);
        require!(signer_ata == self.voting_token_account.key(), VotingError::IncorrectATA);
        require!(self.mint.key() == mint_pk, MintError::NotTheRightMintPK);
        require!(self.treasury_program.key().to_string() == TREASURY_PROGRAM_ID, TreasuryError::NotTheRightTreasuryProgramPK);
        require!(self.voting_tokens_program.key().to_string() == VOTING_TOKENS_PROGRAM_ID, MintError::NotTheRightMintProgramPK);
        require!(treasury_authority_ata == self.treasury_voting_token_account.key(), VotingError::IncorrectTreasuryATA);

        self.add_to_consolidated()?;

        if self.poll.total_for == self.poll.total_against {
            return self.voting_tie();
        }

        let direction: bool = self.poll.total_for > self.poll.total_against;

        let winnings: u64 = calc_winnings_from_votes(
            direction,
            self.voter.direction,
            self.voter.amount,
        );

        if winnings == 0 {
            return Ok(())
        }

        self.reimburse_winnings(winnings)

    }

    fn voting_tie(
        &mut self
    ) -> Result<()> {

        // In the case of a tie everyone gets their votes tokens re-minted
        self.reimburse_votes(self.voting_token_account.to_account_info(), self.voter.amount)

    }

    fn reimburse_votes(
        &self,
        to: AccountInfo<'info_vr>,
        amount: u64
    ) -> Result<()> {

        require!(self.market.state == MarketState::Consolidating, ResultsError::VotingNotFinished);

        let program_account: AccountInfo<'_> = self.voting_tokens_program.to_account_info();

        require!(program_account.key().to_string() == VOTING_TOKENS_PROGRAM_ID, CpiError::WrongProgramID);

        let accounts: MintTokens<'_> = MintTokens{
            payer: self.signer.to_account_info(),
            mint: self.mint.to_account_info(),
            recipient: to,
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            rent: self.rent.to_account_info(),
        };

        let cpi_ctx: CpiContext<'_, '_, '_, '_, MintTokens<'_>> = CpiContext::new(
            program_account,
            accounts,
        );

        mint_tokens(
            cpi_ctx,
            amount,
        )?;

        Ok(())

    }

    fn reimburse_winnings(
        &mut self,
        winnings: u64,
    ) -> Result<()> {
        let cpi_accounts = Transact {
            signer: self.treasury_auth.to_account_info(),                               // This needs to be the treasury authority
            coparty: self.signer.to_account_info(),                                     // This needs to be the person receiving the reimbursement
            treasury: self.treasury.to_account_info(),
            voting_token_account: self.treasury_voting_token_account.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            self.treasury_program.to_account_info(),
            cpi_accounts,
        );

        // Pay out winnings in SOL
        reimburse(
            cpi_ctx,
            winnings,
        )
    }

    fn add_to_consolidated(&mut self) -> Result<()> {

        if self.poll.voters_consolidated.is_some() {

            let mut consolidated_vec: Vec<Pubkey> = self.poll.voters_consolidated.clone().unwrap();
            consolidated_vec.push(self.signer.key());
            
            self.poll.voters_consolidated = Some(consolidated_vec.clone());

        } else {
            
            self.poll.voters_consolidated = Some(Vec::from([self.signer.key()]));

        }

        Ok(())
    }

}
