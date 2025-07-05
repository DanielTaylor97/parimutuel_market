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

use crate::constants::TREASURY_ADDRESS;
use crate::error::{FacetError, MintError, ResultsError, TokenError, TreasuryError, VotingError};
use crate::states::{Market, MarketParams, MarketState, Poll, Voter};
use crate::utils::functions::calc_winnings_from_votes;

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct VoterResult<'info_vr> {
    #[account(mut)]
    pub treasury: Signer<'info_vr>,
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
    pub associated_token_program: Program<'info_vr, AssociatedToken>,
    #[account(mut)]
    pub mint: Account<'info_vr, Mint>,
    pub system_program: Program<'info_vr, System>,
    pub token_program: Program<'info_vr, Token>,
    pub voting_tokens_program: Program<'info_vr, VotingTokens>,
    pub rent: Sysvar<'info_vr, Rent>,
}

impl<'info_vr> VoterResult<'info_vr> {

    pub fn distribute_sol_to_voter(
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
        let treasury_ata: Pubkey = get_associated_token_address(
            &self.treasury.key(),
            &mint_pk,
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
        //  - Treasury should have the expected address                                                         |       √
        //  - ATA needs to be correct                                                                           |       √
        //  - Mint PK needs to be correct                                                                       |       √
        //  - Voting Tokens Program needs to be correct                                                         |       √
        //  - treasury_voting_token_account should be derivable from treasury account                           |       √
        require!(self.market.state == MarketState::Consolidating, ResultsError::VotingNotFinished);
        require!(voters_count_condition, ResultsError::NotAVoter);
        require!(!consolidated_voters_condition, ResultsError::VoterAlreadyConsolidated);
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.market.token == params.authensus_token, TokenError::NotTheSameToken);
        require!(self.treasury.key().to_string() == TREASURY_ADDRESS, TreasuryError::WrongTreasury);
        require!(signer_ata == self.voting_token_account.key(), VotingError::IncorrectATA);
        require!(self.mint.key() == mint_pk, MintError::NotTheRightMintPK);
        require!(self.voting_tokens_program.key() == mint_program_pk, MintError::NotTheRightMintProgramPK);
        require!(treasury_ata == self.treasury_voting_token_account.key(), VotingError::IncorrectTreasuryATA);

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

        // Reset vote amount
        self.voter.amount = 0_u64;

        if winnings == 0 {
            return Ok(())
        }

        self.reimburse_winnings(winnings)

    }

    fn voting_tie(
        &mut self
    ) -> Result<()> {

        // In the case of a tie everyone gets their votes tokens re-minted
        self.reimburse_votes(self.voting_token_account.to_account_info(), self.voter.amount)?;

        // Reset vote amount
        self.voter.amount = 0_u64;

        Ok(())

    }

    fn reimburse_votes(
        &self,
        to: AccountInfo<'info_vr>,
        amount: u64
    ) -> Result<()> {

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
            self.voting_tokens_program.to_account_info(),
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

        let accounts = Transfer {
            from: self.treasury.to_account_info(),
            to: self.signer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, winnings)
        
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
