use std::str::FromStr;
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer}
};
use anchor_spl::{
    associated_token::{get_associated_token_address_with_program_id, AssociatedToken},
    token::TokenAccount
};

use crate::states::Treasury;
use crate::constants::{VOTING_TOKENS_MINT_ID, VOTING_TOKENS_PROGRAM_ID};
use crate::error::TransactionError;

#[derive(Accounts)]
pub struct Transact<'info_t> {
    #[account(mut)]
    pub signer: Signer<'info_t>,
    #[account(mut)]
    pub coparty: Signer<'info_t>,
    #[account(
        mut,
        seeds = [b"treasury"],
        bump,
    )]
    pub treasury: Account<'info_t, Treasury>,
    #[account(mut)]
    pub voting_token_account: Account<'info_t, TokenAccount>,   // This should already be initialised
    pub system_program: Program<'info_t, System>,
    pub associated_token_program: Program<'info_t, AssociatedToken>,
}

impl<'info_t> Transact<'info_t> {
    
    pub fn deposit(
        &mut self,
        amount: u64,
    ) -> Result<()> {

        // Requirements:
        //  - signer should be the treasury authority   √
        require!(self.signer.key() == self.treasury.authority, TransactionError::SignerNotAuthority);

        // Make deposit here
        self.transfer_sol(
            self.coparty.to_account_info(),
            self.treasury.to_account_info(),
            amount,
        )?;

        self.treasury.set_inner(
            Treasury{
                bump: self.treasury.bump,                   // u8
                authority: self.treasury.authority,         // Pubkey
                balance: self.treasury.balance + amount,    // u64
                voting_tokens: self.treasury.voting_tokens, // u64
            }
        );
        
        Ok(())

    }
    
    pub fn reimburse(
        &mut self,
        amount: u64,
    ) -> Result<()> {

        // Requirements:
        //  - signer should be the treasury authority   √
        require!(self.signer.key() == self.treasury.authority, TransactionError::SignerNotAuthority);

        // Make withdrawal here
        self.transfer_sol(
            self.treasury.to_account_info(),
            self.coparty.to_account_info(),
            amount,
        )?;

        self.treasury.set_inner(
            Treasury{
                bump: self.treasury.bump,                   // u8
                authority: self.treasury.authority,         // Pubkey
                balance: self.treasury.balance - amount,    // u64
                voting_tokens: self.treasury.voting_tokens, // u64
            }
        );
        
        Ok(())

    }
    
    pub fn get_balance(&mut self) -> Result<u64> {
        
        Ok(self.treasury.balance)

    }

    fn transfer_sol(
        &self,
        from: AccountInfo<'info_t>,
        to: AccountInfo<'info_t>,
        amount: u64
    ) -> Result<()> {

        let accounts = Transfer {
            from,
            to,
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), accounts);

        transfer(cpi_ctx, amount)

    }

    pub fn receive_voting_tokens(
        &mut self,
        amount: u64,
    ) -> Result<()> {

        let mint_pk: Pubkey = Pubkey::from_str(VOTING_TOKENS_MINT_ID).unwrap();
        let mint_program_pk: Pubkey = Pubkey::from_str(VOTING_TOKENS_PROGRAM_ID).unwrap();

        // ATA of the treasury
        let ata: Pubkey = get_associated_token_address_with_program_id(
            &self.signer.key(),
            &mint_pk,
            &mint_program_pk,
        );

        let token_balance: u64 = self.voting_token_account.amount;

        // Requirements:
        //  - signer should be the treasury authority                                                   √
        //  - voting token account should be the same as the one derived                                √
        //  - ata token balance should be greater than the current treasury balance by exactly `amount` √
        require!(self.signer.key() == self.treasury.authority, TransactionError::SignerNotAuthority);
        require!(self.voting_token_account.key() == ata, TransactionError::WrongATA);
        require!(token_balance == self.treasury.voting_tokens + amount, TransactionError::BalancesDisagree);

        self.treasury.set_inner(
            Treasury {
                bump: self.treasury.bump,                               // u8
                authority: self.treasury.authority,                     // Pubkey
                balance: self.treasury.balance,                         // u64
                voting_tokens: self.treasury.voting_tokens + amount,    // u64
            }
        );
        
        Ok(())

    }

}
