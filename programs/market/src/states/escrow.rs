use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub bump: u8,               // Bump
    pub n_bets: u16,            // The total number of bets that have been placed
    pub bets_consolidated: u16, // The number of bets that have been consolidated once voting is over
    pub tot_for: u64,           // Total amount in normal bets for
    pub tot_against: u64,       // Total amount in normal bets against
    pub tot_underdog: u64,      // Total amount in underdog bets
}
