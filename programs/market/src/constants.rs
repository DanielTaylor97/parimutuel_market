// GENERAL
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;    // Number of Lamports in one SOL
pub const TREASURY_ADDRESS: &str = "authZLKSUerhcGyPPG7fnaqWkEfB9nnAc9JoiSiEy4u";
pub const TREASURY_ADDRESS_BYTES: [u8; 32] = [
    8,  175,    248,    157,    214,    175,    77,     48,     176,    63,     215,    219,    204,    180,    68,     246,
    42, 178,    13,     102,    163,    28,     178,    210,    206,    145,    77,     76,     59,     50,     145,    58,
];

// WAGERS
pub const MAX_ALLOWED_TIMEOUT: i64 = 14*24*60*60*1_000; // 2 weeks (ms)
pub const MIN_ALLOWED_TIMEOUT: i64 = 24*60*60*1_000;    // 1 day (ms)
pub const MAX_WAGERS: u16 = 10_000;                     // Max number of people placing wagers

// VOTING
pub const MAX_VOTE_AMOUNT: u64 = 100*LAMPORTS_PER_SOL;                                      // Max number of votes per voter
pub const MIN_VOTE_AMOUNT: u64 = 1_000_000;                                                 // Min number of votes per voter
pub const VOTE_THRESHOLD: u16 = 1_000;                                                      // Max number of votes in a poll

// CONSOLIDATION
pub const PERCENTAGE_WINNINGS_KEPT: u64 = 95;               // How much of the winnings pot is received by the winning bettors (%)
pub const DIV_BUFFER: u128 = 1_000_000_000;                 // Buffer for arithmetic with uints
pub const MAX_CONSOLIDATION_PERIOD: i64 = 72*60*60*1_000;   // 3 days (ms)
