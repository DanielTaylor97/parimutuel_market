use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount}
};

#[derive(Accounts)]
pub struct MintTokens<'info_m> {
    #[account(mut)]
    pub payer: Signer<'info_m>,
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
        mint::authority = mint,
    )]
    pub mint: Account<'info_m, Mint>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    pub recipient: Account<'info_m, TokenAccount>,
    pub associated_token_program: Program<'info_m, AssociatedToken>,
    pub system_program: Program<'info_m, System>,
    pub token_program: Program<'info_m, Token>,
    pub rent: Sysvar<'info_m, Rent>,
}

impl<'info_m> MintTokens<'info_m> {

    pub fn mint_tokens(
        &mut self,
        bumps: &MintTokensBumps,
        amount: u64,
    ) -> Result<()> {
        
        let seeds: &[&[u8]; 2] = &["mint".as_bytes(), &[bumps.mint]];
        let signer: [&[&[u8]]; 1] = [&seeds[..]];

        let mint_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.mint.to_account_info(),
                to: self.recipient.to_account_info(),
                authority: self.mint.to_account_info(),
            },
            &signer,
        );

        mint_to(
            mint_ctx,
            amount,
        )?;

        msg!("Successfully minted tokens.");

        Ok(())

    }

}
