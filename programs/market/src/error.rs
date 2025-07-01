use anchor_lang::error_code;

#[error_code]
pub enum InitError {

    #[msg("No facets have  been provided for the market to be initialised")]
    NoFacetsProvided,

    #[msg("The betting timeout is larger than the maximum allowed (2 weeks)")]
    TimeoutTooLarge,

    #[msg("The betting timeout is smaller than the minimum allowed (1 day)")]
    TimeoutTooSmall,

}

#[error_code]
pub enum FacetError {

    #[msg("The selected facet is not present in the given market")]
    FacetNotInMarket,

    #[msg("The poll and escrow must be for the same facet")]
    NotTheSameFacet,

}

#[error_code]
pub enum TreasuryError {

    #[msg("The authority of the supplied treasury is not the authority provided")]
    TreasuryAuthoritiesDontMatch,

    #[msg("This is not the expected treasury authority")]
    WrongTreasuryAuthority,

    #[msg("The account supplied is not for the Treasury Program")]
    NotTheRightTreasuryProgramPK,

}

#[error_code]
pub enum MintError {

    #[msg("This is not the expected Voting Tokens Mint")]
    NotTheRightMintPK,

    #[msg("The account supplied is not for the Voting Tokens Program")]
    NotTheRightMintProgramPK,

}

#[error_code]
pub enum MarketError {

    #[msg("The poll/escrow must be for the same market")]
    NotTheSameMarket,

    #[msg("The market is in the wrong state")]
    MarketInWrongState,

}

#[error_code]
pub enum TokenError {

    #[msg("The token address provided does not correspond to the market")]
    NotTheSameToken,

}

#[error_code]
pub enum BettingError {

    #[msg("You have tried to start an ongoing market")]
    StartingWithBetsInPlace,

    #[msg("The bettor needs to be the same person who signs the transaction")]
    SignerDifferentFromBettor,

    #[msg("Not enough funds to place the bet")]
    InsufficientFunds,

    #[msg("The market is not in the betting state right now")]
    MarketNotInBettingState,

    #[msg("There need to be standard wagers placed before an underdog bet can be placed")]
    UnderdogBetTooEarly,

    #[msg("Cannot place underdog bet with regular bet already in place")]
    UnderdogWithOtherBet,

    #[msg("Cannot place another bet when underdog bet is in place")]
    BetWithUnderdogBet,

    #[msg("Too many bettors in the market")]
    TooManyBettors,

}

#[error_code]
pub enum VotingError {

    #[msg("You have tried to start an ongoing poll")]
    StartingWithVotesInPlace,

    #[msg("It is not the voting period yet")]
    NotVotingTime,

    #[msg("The voter needs to come from the same person who signs the transaction")]
    SignerDifferentFromVoter,

    #[msg("Provided ATA does not match the mint and signer")]
    IncorrectATA,

    #[msg("Provided ATA is not for the treasury")]
    IncorrectTreasuryATA,

    #[msg("Cannot vote on a market in which you have already placed bets")]
    CannotVoteWithBets,

    #[msg("Only one vote allowed per market")]
    AlreadyVoted,

    #[msg("The voting has finished for this round")]
    VotingClosed,

    #[msg("Not enough voting tokens to make that transaction")]
    InsufficientVotingTokens,

    #[msg("The mint provided is not the expected mint")]
    IncorrectMint,

    #[msg("Voting amount too low")]
    AmountTooLow,

    #[msg("Voting amount too high")]
    AmountTooHigh,

}

#[error_code]
pub enum ResultsError {

    #[msg("Voting has not yet finished")]
    VotingNotFinished,

    #[msg("Given address is not a bettor in this market-facet combination")]
    NotABettor,

    #[msg("The signer is not the person receiving the results")]
    SignerNotPK,

    #[msg("This person is either not a voter or has already tried to receive their voting reward")]
    NotAVoter,

    #[msg("The wager numbers don't add up")]
    WagersDontAddUp,

    #[msg("The votes don't add up")]
    VotesDontAddUp,

    #[msg("This bettor has already been reimbursed")]
    BettorAlreadyConsolidated,

    #[msg("This voter has already been reimbursed")]
    VoterAlreadyConsolidated,

    #[msg("Not all of the bets have been consolidated")]
    NotAllBetsConsolidated,

    #[msg("Not all of the votes have been consolidated")]
    NotAllVotesConsolidated,

}

#[error_code]
pub enum CpiError {

    #[msg("The program ID for CPI was unexpected")]
    WrongProgramID,

}
