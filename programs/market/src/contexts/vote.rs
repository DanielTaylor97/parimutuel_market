use std::str::FromStr;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{get_associated_token_address_with_program_id, AssociatedToken},
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};

use treasury::{
    self,
    Treasury,
};

use crate::constants::{MAX_VOTE_AMOUNT, MIN_VOTE_AMOUNT, VOTING_TOKENS_MINT_ID, VOTING_TOKENS_PROGRAM_ID, VOTE_THRESHOLD};
use crate::error::{FacetError, MintError, TokenError, VotingError};
use crate::states::{Escrow, Market, MarketParams, MarketState, Poll, Voter};

#[derive(Accounts)]
#[instruction(params: MarketParams)]
pub struct Vote<'info_v> {
    #[account(mut)]
    pub signer: Signer<'info_v>,
    #[account(
        seeds = [b"market", params.authensus_token.as_ref()],
        bump,
    )]
    pub market: Account<'info_v, Market>,
    #[account(
        seeds = [b"escrow", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub escrow: Account<'info_v, Escrow>,
    #[account(
        init_if_needed,
        space = Poll::INIT_SPACE,
        payer = signer,
        seeds = [b"poll", params.authensus_token.as_ref(), params.facet.to_string().as_bytes()],
        bump,
    )]
    pub poll: Account<'info_v, Poll>,
    #[account(
        init_if_needed,
        space = Voter::INIT_SPACE,
        payer = signer,
        seeds = [b"voter", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), signer.key().as_ref()],
        bump,
    )]
    pub voter: Account<'info_v, Voter>,
    #[account(mut)]
    pub voting_token_account: Account<'info_v, TokenAccount>,           // This should already be initialised from wager_results (or purchasing)
    #[account(mut)]
    pub mint: Account<'info_v, Mint>,
    #[account(mut)]
    pub treasury_voting_token_account: Account<'info_v, TokenAccount>,  // This should already be initialised with the treasury
    #[account(mut)]
    pub treasury: Account<'info_v, Treasury>,
    pub system_program: Program<'info_v, System>,
    pub token_program: Program<'info_v, Token>,
    pub associated_token_program: Program<'info_v, AssociatedToken>,
}

impl<'info_v> Vote<'info_v> {
    pub fn add_vote(
        &mut self,
        bumps: &VoteBumps,
        params: &MarketParams,
        amount: u64,
        direction: bool,
    ) -> Result<()> {

        let time: i64 = Clock::get()?.unix_timestamp;

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

        let wagers_count_condition: bool = match self.escrow.bettors.is_some() {
            true => self.escrow.bettors.as_ref().unwrap().contains(&self.signer.key()),
            false => false,
        };

        let voters_count_condition: bool = match self.poll.voters.is_some() {
            true => self.poll.voters.as_ref().unwrap().contains(&self.signer.key()),
            false => false,
        };

        // Requirements:                                                                |   Implemented:
        //  - The token must be the same as that which instantiated the market          |       √
        //  - treasury_voting_token_account should be derivable from treasury authority |       √
        //  - The betting round has finished                                            |       √
        //  - Cannot have voted here already                                            |       √
        //  - Voting threshold cannot have been reached yet                             |       √
        //  - ATA needs to be correct                                                   |       √
        //  - ATA must have sufficient tokens for this vote                             |       √
        //  - Vote amount must be higher than minimum                                   |       √
        //  - Vote amount must be lower than maximum                                    |       √
        //  - Mint provided must be correct                                             |       √
        //  - Voter cannot have placed any bets                                         |       √
        //  - Market should contain the given facet                                     |       √
        //  - Mint PK needs to be correct                                               |       √
        require!(self.market.token == params.authensus_token, TokenError::NotTheSameToken);
        require!(treasury_authority_ata == self.treasury_voting_token_account.key(), VotingError::IncorrectTreasuryATA);
        require!(self.market.start_time + self.market.timeout < time, VotingError::NotVotingTime);
        require!(!voters_count_condition, VotingError::AlreadyVoted);
        require!(self.poll.total_for + self.poll.total_against < VOTE_THRESHOLD.into(), VotingError::VotingClosed);    // Better to do time- or threshold-based?
        require!(signer_ata == self.voting_token_account.key(), VotingError::IncorrectATA);
        require!(self.voting_token_account.amount >= amount, VotingError::InsufficientVotingTokens);
        require!(amount >= MIN_VOTE_AMOUNT, VotingError::AmountTooLow);
        require!(amount <= MAX_VOTE_AMOUNT, VotingError::AmountTooHigh);
        require!(self.mint.key() == mint_pk, VotingError::IncorrectMint);
        require!(!wagers_count_condition, VotingError::CannotVoteWithBets);
        require!(self.market.facets.contains(&params.facet), FacetError::FacetNotInMarket);
        require!(self.mint.key() == mint_pk, MintError::NotTheRightMintPK);

        // If the market state is still set to Betting but the betting markets have passed the timeout, then change to Voting
        if self.market.state == MarketState::Betting && self.market.start_time + self.market.timeout < time {
            self.market.set_inner(
                Market {
                    bump: self.market.bump,             // u8
                    token: self.market.token,           // Pubkey
                    facets: self.market.facets.clone(), // Vec<Facet>
                    start_time: self.market.start_time, // i64
                    timeout: self.market.timeout,       // i64
                    state: MarketState::Voting,         // MarketState
                    round: self.market.round,           // u16
                }
            );
        }

        // Receive voting tokens from ATA
        self.receive_vote_token_into_treasury(self.voting_token_account.to_account_info(), amount)?;

        // Update poll + voter totals
        // Amount of vote does not change number of votes in the poll, only redemption
        // Everyone is marked as a single vote in the poll
        let vote_for: u64 = match direction {
            true => 1_u64,
            false => 0_u64
        };
        
        let vote_against: u64 = 1 - vote_for;

        // Initialise the poll if necessary
        if self.poll.total_for + self.poll.total_against == 0 {
            self.poll.set_inner(
                Poll {
                    bump: bumps.poll,                               // u8
                    market: params.authensus_token,                 // Pubkey
                    facet: params.facet.clone(),                    // Facet
                    voters: Some(Vec::from([self.signer.key()])),   // Option<Vec<Pubkey>>
                    voters_consolidated: None,                      // Option<Vec<Pubkey>>
                    total_for: vote_for,                            // u64
                    total_against: vote_against,                    // u64
                }
            );
        } else {
            let voters: &mut Vec<Pubkey> = &mut self.poll.voters.clone().unwrap();
            voters.push(self.signer.key());

            self.poll.set_inner(
                Poll {
                    bump: self.poll.bump,                                   // u8
                    market: self.poll.market,                               // Pubkey
                    facet: self.poll.facet.clone(),                         // Facet
                    voters: Some(voters.clone()),                           // Option<Vec<Pubkey>>
                    voters_consolidated: None,                              // Option<Vec<Pubkey>>
                    total_for: self.poll.total_for + vote_for,              // u64
                    total_against: self.poll.total_against + vote_against,  // u64
                }
            );
        }

        // As per requirements above, voter cannot have already cast a vote; so this is de novo
        self.voter.set_inner(
            Voter {
                bump: bumps.voter,              // u8
                pk: self.signer.key(),             // Pubkey
                market: params.authensus_token, // Pubkey
                facet: params.facet.clone(),    // Facet
                amount,                         // u64
                direction,                      // bool
            }
        );
        
        Ok(())

    }

    fn receive_vote_token_into_treasury(
        &self,
        from: AccountInfo<'info_v>,
        amount: u64
    ) -> Result<()> {

        let accounts = TransferChecked {
            mint: self.mint.to_account_info(),
            from,
            to: self.treasury_voting_token_account.to_account_info(),   // This should be the ATA for the treasury
            authority: self.signer.to_account_info(),                   // Owner of the source token account
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), accounts);

        transfer_checked(
            cpi_ctx,
            amount,
            self.mint.decimals,
        )?;

        Ok(())

    }

}
