use anchor_lang::prelude::{borsh::{BorshSerialize, BorshDeserialize}, *};

#[account]
pub struct Market {
    pub bump: u8,
    pub token: Pubkey,
    pub facets: Vec<Facet>,
    pub state: MarketState,
    pub round: u16,
}

impl Space for Market {
    // Discriminator (8) + bumps (8) + token (32) + facets(?) + state (?) + round (2)
    const INIT_SPACE: usize = 8 + 1 + 32 + 2;   // INCOMPLETE
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum MarketState {
    Initialised,
    Inactive,
    Betting,
    Voting,
    Consolidating,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
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
