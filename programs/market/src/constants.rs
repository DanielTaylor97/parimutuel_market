// WAGERS
pub const MAX_ALLOWED_TIMEOUT: i64 = 14*24*60*60*1_000;   // 2 weeks (ms)
pub const MAX_WAGERS: u16 = 10_000;

// VOTING
pub const MAX_VOTES: u16 = 1_000;                                                           // Max number of votes per voter
pub const VOTE_THRESHOLD: u64 = 1_000;                                                      // Max number of votes in a poll
pub const VOTING_TOKENS_PROGRAM_ID: &str = "8MrQHajcffRco93T4kR5FiLnrCYA7nj1yYXoauHRdg5d";  // 
pub const VOTING_TOKENS_MINT_ID: &str = "mintpubkey";                                       // 

// CONSOLIDATION
pub const PERCENTAGE_WINNINGS_KEPT: u64 = 95;   // How much of the winnings pot is received by the winning bettors (%)
pub const DIV_BUFFER: u64 = 1_000_000;          // Buffer for arithmetic with uints
