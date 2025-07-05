#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

declare_id!("i7B3be3uBkdiXuWaR7oNopETpKdFYKjVaDBGiVwJiCd");

pub mod states;
pub mod contexts;
pub mod error;
pub mod constants;

pub use states::*;
pub use contexts::*;
pub use error::*;

#[program]
pub mod treasury_program {
    use super::*;

    pub fn initialise(ctx: Context<Initialise>) -> Result<()> {
        
        ctx.accounts.initialise(&ctx.bumps)

    }

    pub fn deposit(
        ctx: Context<Transact>,
        amount: u64,
    ) -> Result<()> {

        ctx.accounts.deposit(amount)

    }

    pub fn reimburse(
        ctx: Context<Transact>,
        amount: u64,
    ) -> Result<()> {

        ctx.accounts.reimburse(amount)

    }

    pub fn get_sol_balance(
        ctx: Context<Transact>,
    ) -> Result<u64> {

        ctx.accounts.get_sol_balance()

    }

    pub fn get_voting_token_balance(
        ctx: Context<Transact>,
    ) -> Result<u64> {

        ctx.accounts.get_voting_token_balance()

    }

}
