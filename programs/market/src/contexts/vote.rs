use std::str::FromStr;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{get_associated_token_address_with_program_id, AssociatedToken},
    token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked},
};

use crate::constants::{VOTING_TOKENS_MINT_ID, VOTING_TOKENS_PROGRAM_ID, VOTE_THRESHOLD};
use crate::error::VotingError;
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
        seeds = [b"voter", params.authensus_token.as_ref(), params.facet.to_string().as_bytes(), params.address.as_ref()],
        bump,
    )]
    pub voter: Account<'info_v, Voter>,
    #[account(mut)]
    pub mint: Account<'info_v, Mint>,
    #[account(mut)]
    pub voting_token_account: Account<'info_v, TokenAccount>,   // This should already be initialised from wager_results (or purchasing)
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

        // // This condition will also need to change if we decide to do time-based poll closing
        // if self.poll.total_for + self.poll.total_against < VOTE_THRESHOLD {
        //     self.market.set_inner(
        //         Market {
        //             bump: self.market.bump,             // u8
        //             token: self.market.token,           // Pubkey
        //             facets: self.market.facets.clone(), // Vec<Facet>
        //             state: MarketState::Inactive,       // MarketState
        //             round: self.market.round,           // u16
        //         }
        //     );

        //     return Err(anchor_lang::error!(VotingError::VotingClosed));
        // }

        let mint_pk = Pubkey::from_str(VOTING_TOKENS_MINT_ID).unwrap();
        let mint_program_pk = Pubkey::from_str(VOTING_TOKENS_PROGRAM_ID).unwrap();

        let signer_ata = get_associated_token_address_with_program_id(
             &self.signer.key(),
             &mint_pk,
             &mint_program_pk,
        );

        // Requirements:
        //  - The betting round has finished                √
        //  - Cannot have voted here already                √
        //  - Voting threshold cannot have been reached yet √
        //  - signer needs to be the same as the voter pk   √
        //  - ATA needs to be correct                       √
        //  - ATA must have sufficient tokens for this vote √
        //  - Mint provided must be correct
        require!(self.escrow.end_time < time, VotingError::NotVotingTime);
        require!(!self.poll.voters.as_ref().unwrap().contains(&params.address), VotingError::AlreadyVoted);
        require!(self.poll.total_for + self.poll.total_against < VOTE_THRESHOLD, VotingError::VotingClosed);   // Better to do time- or threshold-based?
        require!(self.signer.key() == self.voter.pk, VotingError::SignerDifferentFromVoter);
        require!(signer_ata == self.voting_token_account.key(), VotingError::IncorrectATA);
        require!(self.voting_token_account.amount >= amount, VotingError::InsufficientVotingTokens);
        require!(self.mint.key() == mint_pk, VotingError::IncorrectMint);

        if self.market.state != MarketState::Voting && self.escrow.end_time < time {
            self.market.set_inner(
                Market {
                    bump: self.market.bump,             // u8
                    token: self.market.token,           // Pubkey
                    facets: self.market.facets.clone(), // Vec<Facet>
                    state: MarketState::Voting,         // MarketState
                    round: self.market.round,           // u16
                }
            );
        }

        require!(self.market.state == MarketState::Voting, VotingError::NotVotingTime);

        // Receive voting tokens from ATA
        self.receive_vote_token(self.voting_token_account.to_account_info(), amount)?;

        // Update poll + voter
        let total_for: u64 = match direction {
            true => amount,
            false => 0_u64
        };
        
        let total_against: u64 = amount - total_for;

        if self.poll.total_for + self.poll.total_against == 0 {
            self.poll.set_inner(
                Poll {
                    bump: bumps.poll,                           // u8
                    market: params.authensus_token,             // Pubkey
                    facet: self.escrow.facet.clone(),           // Facet
                    voters: Some(Vec::from([params.address])),  // Option<Vec<Pubkey>>
                    total_for,                                  // u64
                    total_against,                              // u64
                }
            );
        } else {
            let voters: &mut Vec<Pubkey> = &mut self.poll.voters.clone().unwrap();
            voters.push(params.address);

            self.poll.set_inner(
                Poll {
                    bump: self.poll.bump,                                   // u8
                    market: self.poll.market,                               // Pubkey
                    facet: self.poll.facet.clone(),                         // Facet
                    voters: Some(voters.clone()),                           // Option<Vec<Pubkey>>
                    total_for: self.poll.total_for + total_for,             // u64
                    total_against: self.poll.total_against + total_against, // u64
                }
            );
        }

        self.voter.set_inner(
            Voter {
                bump: bumps.voter,              // u8
                pk: params.address,             // Pubkey
                market: params.authensus_token, // Pubkey
                facet: params.facet.clone(),    // Facet
                amount,                         // u64
                direction,                      // bool
            }
        );
        
        Ok(())

    }

    fn receive_vote_token(
        &self,
        from: AccountInfo<'info_v>,
        amount: u64
    ) -> Result<()> {

        let accounts = TransferChecked {
            mint: self.mint.to_account_info(),
            from,
            to: self.voting_token_account.to_account_info(),
            authority: self.signer.to_account_info(),           // Owner of the source token account
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
