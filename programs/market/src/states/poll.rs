use anchor_lang::prelude::*;

use super::Facet;

#[account]
pub struct Poll {
    pub bump: u8,                       // Bump
    pub market: Pubkey,                 // The pubkey of the market account
    pub facet: Facet,                   // The facet for which the poll exists within the market
    pub voters: Option<Vec<Pubkey>>,    // Everyone who has placed a vote in the poll
    pub total_for: u64,                 // Total votes for facet
    pub total_against: u64,             // Total votes against facet
    // pub total_underdog: u64,            // Total amount in underdog bets
}

impl Space for Poll {
    // Discriminator (8) + bump(1) + market (32) + facet (?) + voters (?) + total_for (8) + total_against (8)
    const INIT_SPACE: usize = 8 + 1 + 32 + 8 + 8;   // ICOMPLETE
}
