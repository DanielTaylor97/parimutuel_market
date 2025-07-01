use anchor_lang::prelude::*;

use super::Facet;

#[account]
#[derive(InitSpace)]
pub struct Voter {
    pub bump: u8,
    pub pk: Pubkey,
    pub market: Pubkey,
    pub facet: Facet,
    pub amount: u64,
    pub direction: bool,
}

// impl Space for Voter {
//     // Discriminator (8) + pk (32) + market (32) + facet (?) + amount (8) + direction (1)
//     const INIT_SPACE: usize = 8 + 32 + 32 + 8 + 1;  // INCOMPLETE
// }
