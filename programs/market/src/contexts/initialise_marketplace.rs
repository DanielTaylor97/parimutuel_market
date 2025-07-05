use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use voting_tokens::{
    self,
    id as get_voting_tokens_program_id,
};

use crate::constants::TREASURY_ADDRESS;
use crate::error::{MintError, TreasuryError};

#[derive(Accounts)]
pub struct InitialiseMarketplace<'info_i> {
    #[account(mut)]
    pub treasury: SystemAccount<'info_i>,
    #[account(
        init,
        payer = treasury,
        associated_token::mint = mint,
        associated_token::authority = treasury,
    )]
    pub treasury_token_account: Account<'info_i, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info_i, Mint>,
    pub token_program: Program<'info_i, Token>,
    pub associated_token_program: Program<'info_i, AssociatedToken>,
    pub system_program: Program<'info_i, System>,
}

impl<'info_i> InitialiseMarketplace<'info_i> {

    pub fn init_marketplace(
        &mut self,
    ) -> Result<()> {

        let mint_program_pk: Pubkey = get_voting_tokens_program_id();
        let mint_pk: Pubkey= Pubkey::find_program_address(
            &[b"mint"],
            &mint_program_pk,
        ).0;
        
        // Requirements:                                    |   Implemented:
        //  - Mint must be the voting tokens mint           |       √
        //  - Treasury must be the expected address         |       √
        require!(self.mint.key() == mint_pk, MintError::NotTheRightMintPK);
        require!(self.treasury.key().to_string() == TREASURY_ADDRESS, TreasuryError::WrongTreasury);

        Ok(())

    }
}
