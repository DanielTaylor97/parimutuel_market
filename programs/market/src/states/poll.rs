use anchor_lang::prelude::*;

use super::Facet;
use crate::constants::VOTE_THRESHOLD;

#[account]
#[derive(InitSpace)]
pub struct Poll {
    pub bump: u8,                                   // Bump
    pub market: Pubkey,                             // The pubkey of the market account
    pub facet: Facet,                               // The facet for which the poll exists within the market
    #[max_len(VOTE_THRESHOLD)]
    pub voters: Option<Vec<Pubkey>>,                // Everyone who has placed a vote in the poll
    #[max_len(VOTE_THRESHOLD)]
    pub voters_consolidated: Option<Vec<Pubkey>>,   // Count of the number of voters whose winnings have been calculated and reimbursed
    pub total_for: u16,                             // Total votes for facet
    pub total_against: u16,                         // Total votes against facet
}
