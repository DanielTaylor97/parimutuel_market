use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Treasury {
    pub bump: u8,           // Bump
    pub authority: Pubkey,      // Payer of the initialisation and transactions
    pub balance: u64,       // Total SOL in the treasury
    pub voting_tokens: u64, // Number of voting tokens spent in the markets -- this number can only go up
}
