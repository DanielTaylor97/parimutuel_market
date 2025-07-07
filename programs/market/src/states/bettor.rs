use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Bettor {
    pub bump: u8,
    pub tot_for: u64,
    pub tot_against: u64,
    pub tot_underdog: u64,
    pub bets_signed: Option<[u8; 64]>,
}
