#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

declare_id!("8MrQHajcffRco93T4kR5FiLnrCYA7nj1yYXoauHRdg5d");

pub mod constants;
pub mod error;
pub mod contexts;

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
