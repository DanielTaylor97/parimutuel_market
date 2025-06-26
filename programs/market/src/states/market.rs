use anchor_lang::prelude::{borsh::{BorshSerialize, BorshDeserialize}, *};

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub bump: u8,
    pub token: Pubkey,
    #[max_len(8)]
    pub facets: Vec<Facet>,
    pub state: MarketState,
    pub round: u16,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, InitSpace, PartialEq)]
pub enum MarketState {
    Initialised,
    Inactive,
    Betting,
    Voting,
    Consolidating,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, InitSpace)]
pub struct MarketParams {
    pub authensus_token: Pubkey,
    pub facet: Facet,
    pub address: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, InitSpace, PartialEq)]
pub enum Facet {
    Truthfulness,
    Originality,
    Authenticity,
    // TBC
}

impl ToString for Facet {
    fn to_string(&self) -> String {
        match self {
            Facet::Truthfulness => "truthfulness".to_string(),
            Facet::Originality => "originality".to_string(),
            Facet::Authenticity => "authenticity".to_string(),
        }
    }
}
