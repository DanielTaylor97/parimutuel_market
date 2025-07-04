// GENERAL
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;    // Number of Lamports in one SOL

// WAGERS
pub const MAX_ALLOWED_TIMEOUT: i64 = 14*24*60*60*1_000; // 2 weeks (ms)
pub const MIN_ALLOWED_TIMEOUT: i64 = 24*60*60*1_000;    // 1 day (ms)
pub const MAX_WAGERS: u16 = 10_000;                     // Max number of people placing wagers

// VOTING
pub const MAX_VOTE_AMOUNT: u64 = 100*LAMPORTS_PER_SOL;                                      // Max number of votes per voter
pub const MIN_VOTE_AMOUNT: u64 = 1_000_000;                                                 // Min number of votes per voter
pub const VOTE_THRESHOLD: u16 = 1_000;                                                      // Max number of votes in a poll

// CONSOLIDATION
pub const PERCENTAGE_WINNINGS_KEPT: u64 = 95;   // How much of the winnings pot is received by the winning bettors (%)
pub const DIV_BUFFER: u128 = 1_000_000_000;     // Buffer for arithmetic with uints
