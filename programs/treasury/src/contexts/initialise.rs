use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, TokenAccount, Token}
};

use voting_tokens::id as voting_tokens_program_id;

use crate::states::Treasury;
use crate::error::InitError;

#[derive(Accounts)]
pub struct Initialise<'info_i> {
    #[account(mut)]
    pub signer: Signer<'info_i>,
    #[account(
        init,
        space = 8 + Treasury::INIT_SPACE,
        payer = signer,
        seeds = [b"treasury"],
        bump,
    )]
    pub treasury: Box<Account<'info_i, Treasury>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = treasury,
    )]
    pub voting_token_account: Account<'info_i, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info_i, Mint>,
    pub token_program: Program<'info_i, Token>,
    pub associated_token_program: Program<'info_i, AssociatedToken>,
    pub system_program: Program<'info_i, System>,
}

impl<'info_i> Initialise<'info_i> {

    pub fn initialise(
        &mut self,
        bumps: &InitialiseBumps,
    ) -> Result<()> {

        let mint_program_pk: Pubkey = voting_tokens_program_id();
        let expected_mint_address: Pubkey = Pubkey::find_program_address(
            &[b"mint"],
            &mint_program_pk
        ).0;

        // Requirements:                            |   Implemented:
        //  - The mint must be the expected account |       √
        require!(self.mint.key() == expected_mint_address, InitError::WrongTokenMint);

        self.treasury.set_inner(
            Treasury { 
                bump: bumps.treasury,           //u8
                authority: self.signer.key(),   // Pubkey
            }
        );

        msg!("Treasury successfully initialised with key {:?}", self.treasury.key().to_string());
        
        Ok(())

    }

}
