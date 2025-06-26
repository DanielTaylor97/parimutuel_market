use anchor_lang::prelude::*;

#[account]
pub struct Treasury {
    pub bump: u8,               // Bump
    pub total_minted: u64,      // Total number of tokens minted
    pub total_in_vault: u64,    // Total number of minted tokens in the vault
    pub balance: u64,           // Total balance of SOL
}

impl Space for Treasury {
    // Discriminator (8) + total_minted (8) + total_in_vault (8) + balance (8)
    const INIT_SPACE: usize = 8 + 8 + 8 + 8;
}
