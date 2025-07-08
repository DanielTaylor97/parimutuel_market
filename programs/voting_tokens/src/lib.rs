#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

declare_id!("2VBwYa8nVPvMXo8tundt6QmzybmExHzDFGRNiLjn3K6p");

pub mod constants;
pub mod error;
pub mod contexts;

pub use constants::*;
pub use error::*;
pub use contexts::*;

#[program]
pub mod voting_tokens {
    use super::*;

    pub fn init(ctx: Context<Initialise>, params: InitTokenParams) -> Result<()> {
        
        ctx.accounts.init(
            &ctx.bumps,
            params,
        )
    
    }

    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        
        ctx.accounts.mint_tokens(
            &ctx.bumps,
            amount,
            // params,
        )
    
    }

}
