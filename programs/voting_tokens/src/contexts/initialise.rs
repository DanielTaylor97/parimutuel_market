use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{
        create_metadata_accounts_v3,
        mpl_token_metadata::types::DataV2,
        CreateMetadataAccountsV3, 
        Metadata as Metaplex,
    },
    token::{Mint, Token},
};

use crate::constants::TREASURY_ADDRESS;
use crate::error::InitError;

#[derive(Accounts)]
#[instruction(params: InitTokenParams)]
pub struct Initialise<'info_i> {
    #[account(mut)]
    pub signer: Signer<'info_i>,
    #[account(
        init,
        payer = signer,
        seeds = [b"mint"],
        bump,
        mint::decimals = params.decimals,
        mint::authority = mint
    )]
    pub mint: Account<'info_i, Mint>,
    /// CHECK: Metaplex account that will be checked by the mpl program
    #[account(mut)]
    pub metadata: UncheckedAccount<'info_i>,
    pub system_program: Program<'info_i, System>,
    pub token_program: Program<'info_i, Token>,
    pub token_metadata_program: Program<'info_i, Metaplex>,
    pub rent: Sysvar<'info_i, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitTokenParams {
    name: String,
    symbol: String,
    uri: String,
    decimals: u8,
}

impl<'info_i> Initialise<'info_i> {

    pub fn init(
        &mut self,
        bumps: &InitialiseBumps,
        metadata: InitTokenParams,
    ) -> Result<()> {

        // Requirements:                        |   Implemented:
        //  - Signer should be the treasury     |       √
        //  - Name is `AuthensusVotingToken`    |       √
        //  - Symbol is `AUTHVOTE`              |       √
        //  - URI should be empty               |       √
        //  - Decimals should be 9              |       √
        require!(self.signer.key().to_string() == TREASURY_ADDRESS, InitError::WrongSigner);
        require!(metadata.name == "AuthensusVotingToken".to_string(), InitError::WrongName);
        require!(metadata.symbol == "AUTHVOTE".to_string(), InitError::WrongSymbol);
        require!(metadata.uri == "".to_string(), InitError::WrongUri);
        require!(metadata.decimals == 9, InitError::WrongDecimals);

        let seeds: &[&[u8]; 2] = &["mint".as_bytes(), &[bumps.mint]];
        let signer: [&[&[u8]]; 1] = [&seeds[..]];

        let token_data = DataV2{
            name: metadata.name,
            symbol: metadata.symbol,
            uri: metadata.uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let metadata_ctx = CpiContext::new_with_signer(
            self.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                payer: self.signer.to_account_info(),
                update_authority: self.mint.to_account_info(),
                mint: self.mint.to_account_info(),
                metadata: self.metadata.to_account_info(),
                mint_authority: self.mint.to_account_info(),
                system_program: self.system_program.to_account_info(),
                rent: self.rent.to_account_info(),
            },
            &signer,
        );

        create_metadata_accounts_v3(
            metadata_ctx,
            token_data,
            false,
            true,
            None,
        )?;

        msg!("Token mint created successfully with key {:?}", self.mint.key().to_string());

        Ok(())

    }
    
}
