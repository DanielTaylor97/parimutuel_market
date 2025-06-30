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
    pub tot_for: u64,                   // Total amount in normal bets for
    pub tot_against: u64,               // Total amount in normal bets against
    pub tot_underdog: u64,              // Total amount in underdog bets
}
