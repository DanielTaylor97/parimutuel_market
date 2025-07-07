use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Poll {
    pub bump: u8,                   // Bump
    pub total_for: u16,             // Total votes for facet
    pub total_against: u16,         // Total votes against facet
    pub total_consolidated: u16,    // Number of votes consolidated after voting is finished
}
