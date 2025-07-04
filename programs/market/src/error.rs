use anchor_lang::error_code;

#[error_code]
pub enum InitError {

    #[msg("No facets have  been provided for the market to be initialised")]
    NoFacetsProvided = 0,

    #[msg("The betting timeout is larger than the maximum allowed (2 weeks)")]
    TimeoutTooLarge = 1,

    #[msg("The betting timeout is smaller than the minimum allowed (1 day)")]
    TimeoutTooSmall = 2,

}

#[error_code]
pub enum FacetError {

    #[msg("The selected facet is not present in the given market")]
    FacetNotInMarket = 100,

    #[msg("The poll and escrow must be for the same facet")]
    NotTheSameFacet = 101,

}

#[error_code]
pub enum TreasuryError {

    #[msg("The authority of the supplied treasury is not the authority provided")]
    TreasuryAuthoritiesDontMatch = 200,

    #[msg("This is not the expected treasury")]
    WrongTreasury = 201,

    #[msg("The account supplied is not for the Treasury Program")]
    NotTheRightTreasuryProgramPK = 202,

}

#[error_code]
pub enum MintError {

    #[msg("This is not the expected Voting Tokens Mint")]
    NotTheRightMintPK = 300,

    #[msg("The account supplied is not for the Voting Tokens Program")]
    NotTheRightMintProgramPK = 301,

}

#[error_code]
pub enum MarketError {

    #[msg("The poll/escrow must be for the same market")]
    NotTheSameMarket = 400,

    #[msg("The market is in the wrong state")]
    MarketInWrongState = 401,

}

#[error_code]
pub enum TokenError {

    #[msg("The token address provided does not correspond to the market")]
    NotTheSameToken = 500,

}

#[error_code]
pub enum BettingError {

    #[msg("You have tried to start an ongoing market")]
    StartingWithBetsInPlace = 600,

    #[msg("The bettor needs to be the same person who signs the transaction")]
    SignerDifferentFromBettor = 601,

    #[msg("Not enough funds to place the bet")]
    InsufficientFunds = 602,

    #[msg("The market is not in the betting state right now")]
    MarketNotInBettingState = 603,

    #[msg("There need to be standard wagers placed before an underdog bet can be placed")]
    UnderdogBetTooEarly = 604,

    #[msg("Cannot place underdog bet with regular bet already in place")]
    UnderdogWithOtherBet = 605,

    #[msg("Cannot place another bet when underdog bet is in place")]
    BetWithUnderdogBet = 606,

    #[msg("Too many bettors in the market")]
    TooManyBettors = 607,

}

#[error_code]
pub enum VotingError {

    #[msg("You have tried to start an ongoing poll")]
    StartingWithVotesInPlace = 700,

    #[msg("It is not the voting period yet")]
    NotVotingTime = 701,

    #[msg("The voter needs to come from the same person who signs the transaction")]
    SignerDifferentFromVoter = 702,

    #[msg("Provided ATA does not match the mint and signer")]
    IncorrectATA = 703,

    #[msg("Provided ATA is not for the treasury")]
    IncorrectTreasuryATA = 704,

    #[msg("Cannot vote on a market in which you have already placed bets")]
    CannotVoteWithBets = 705,

    #[msg("Only one vote allowed per market")]
    AlreadyVoted = 706,

    #[msg("The voting has finished for this round")]
    VotingClosed = 707,

    #[msg("Not enough voting tokens to make that transaction")]
    InsufficientVotingTokens = 708,

    #[msg("The mint provided is not the expected mint")]
    IncorrectMint = 709,

    #[msg("Voting amount too low")]
    AmountTooLow = 710,

    #[msg("Voting amount too high")]
    AmountTooHigh = 711,

}

#[error_code]
pub enum ResultsError {

    #[msg("Voting has not yet finished")]
    VotingNotFinished = 800,

    #[msg("Given address is not a bettor in this market-facet combination")]
    NotABettor = 801,

    #[msg("The signer is not the person receiving the results")]
    SignerNotPK = 802,

    #[msg("This person is either not a voter or has already tried to receive their voting reward")]
    NotAVoter = 803,

    #[msg("The wager numbers don't add up")]
    WagersDontAddUp = 804,

    #[msg("The votes don't add up")]
    VotesDontAddUp = 805,

    #[msg("This bettor has already been reimbursed")]
    BettorAlreadyConsolidated = 806,

    #[msg("This voter has already been reimbursed")]
    VoterAlreadyConsolidated = 807,

    #[msg("Not all of the bets have been consolidated")]
    NotAllBetsConsolidated = 808,

    #[msg("Not all of the votes have been consolidated")]
    NotAllVotesConsolidated = 809,

}

#[error_code]
pub enum CpiError {

    #[msg("The program ID for CPI was unexpected")]
    WrongProgramID = 900,

}
