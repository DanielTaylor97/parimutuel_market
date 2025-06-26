use anchor_lang::prelude::*;

use crate::states::Facet;
use crate::constants::MAX_WAGERS;

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub bump: u8,                       // Bump
    pub initialiser: Pubkey,            // The pubkey of the person who initialised this round of the market
    pub market: Pubkey,                 // The pubkey of the market account
    pub facet: Facet,                   // The facet for which the escrow exists within the market
    #[max_len(MAX_WAGERS)]
    pub bettors: Option<Vec<Pubkey>>,   // Everyone who has placed a bet in escrow
    pub start_time: i64,                // Time at which the market started
    pub end_time: i64,                  // Time at which the market ends
    pub tot_for: u64,                   // Total amount in normal bets for
    pub tot_against: u64,               // Total amount in normal bets against
    pub tot_underdog: u64,              // Total amount in underdog bets
}

// impl Space for Escrow {
//     // Discriminator (8) + bump (8) + initialiser (32) + market (32) + facet (?) + bettors (?) + start_time (16) + tot_for (8) + tot_against (8) + tot_underdog (8)
//     const INIT_SPACE: usize = 8 + 1 + 32 + 32 + 16 + 8 + 8 + 8; // INCOMPLETE
// }
