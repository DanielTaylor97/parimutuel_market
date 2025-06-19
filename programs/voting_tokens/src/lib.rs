use anchor_lang::prelude::*;

declare_id!("8MrQHajcffRco93T4kR5FiLnrCYA7nj1yYXoauHRdg5d");

#[program]
pub mod voting_tokens {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
