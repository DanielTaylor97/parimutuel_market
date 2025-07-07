use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub bump: u8,                                   // Bump
    pub tot_for: u64,                               // Total amount in normal bets for
    pub tot_against: u64,                           // Total amount in normal bets against
    pub tot_underdog: u64,                          // Total amount in underdog bets
}
