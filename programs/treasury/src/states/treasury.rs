use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Treasury {
    pub bump: u8,           // Bump
    pub authority: Pubkey,  // Payer of the initialisation and transactions
}
