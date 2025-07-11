use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Voter {
    pub bump: u8,
    pub amount: u64,
    pub direction: bool,
    pub vote_signed: Option<[u8; 64]>,
}
