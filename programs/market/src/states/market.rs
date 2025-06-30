use anchor_lang::prelude::{borsh::{BorshSerialize, BorshDeserialize}, *};

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub bump: u8,           // Bump
    pub token: Pubkey,      // Authensus token to which the market corresponds
    #[max_len(8)]
    pub facets: Vec<Facet>, // Vector of Facets around which wagers can be made and votes must be cast
    pub start_time: i64,    // Time at which the most recent wagers markets started
    pub timeout: i64,       // Total time for which the wagers markets will operate
    pub state: MarketState, // Current state of the market
    pub round: u16,         // Number of this round of the market
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
