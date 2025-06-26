use anchor_lang::prelude::*;

use crate::states::Facet;

#[account]
#[derive(InitSpace)]
pub struct Bettor {
    pub bump: u8,
    pub pk: Pubkey,
    pub market: Pubkey,
    pub facet: Facet,
    pub tot_for: u64,
    pub tot_against: u64,
    pub tot_underdog: u64,
}

// impl Space for Bettor {
//     // Discriminator (8) + pk (32) + market (32) + facet (?) + tot_for (8) + tot_against (8) + tot_underdog (8)
//     const INIT_SPACE: usize = 8 + 32 + 32 + 8 + 8 + 8;  // INCOMPLETE
// }
