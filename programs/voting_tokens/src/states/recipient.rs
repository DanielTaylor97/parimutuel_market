use anchor_lang::prelude::*;

#[account]
pub struct Recipient {
    pub bump: u8,               // Bump
    pub total_minted: u64,      // Total number of tokens minted
    pub total_in_vault: u64,    // Total number of minted tokens in the vault
    pub balance: u64,           // Total balance of SOL
}

impl Space for Recipient {
    // Discriminator (8) + 
    const INIT_SPACE: usize = 8;
}
