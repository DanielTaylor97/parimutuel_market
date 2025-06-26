use anchor_lang::error_code;

#[error_code]
pub enum FacetError {
    #[msg("The selected facet is not present in the given market")]
    FacetNotInMarket,
}

#[error_code]
pub enum BettingError {

    #[msg("Not enough funds to place the bet")]
    InsufficientFunds,

    #[msg("The market is not in the betting state right now")]
    MarketNotInBettingState,

    #[msg("The betting timeout is larger than the maximum allowed (2 weeks)")]
    TimeoutTooLarge,

    #[msg("There need to be standard wagers placed before an underdog bet can be placed")]
    UnderdogBetTooEarly,

    #[msg("Cannot place underdog bet with regular bet already in place")]
    UnderdogWithOtherBet,

    #[msg("Cannot place another bet when underdog bet is in place")]
    BetWithUnderdogBet,

}

#[error_code]
pub enum VotingError {

    #[msg("It is not the voting period yet")]
    NotVotingTime,

    #[msg("Only one vote allowed per market")]
    AlreadyVoted,

    #[msg("The voting has finished for this round")]
    VotingClosed,

}

#[error_code]
pub enum ResultsError {

    #[msg("Voting has not yet finished")]
    VotingNotFinished,

    #[msg("Given address is not a bettor in this market-facet combination")]
    NotABettor,

    #[msg("This person is either not a voter or has already tried to receive their voting reward")]
    NotAVoter,

    // #[msg("The underdog bets have not yet been resolved")]
    // UnderdogBetsNotResolved,

    #[msg("The wager numbers don't add up")]
    WagersDontAddUp,

    #[msg("The votes don't add up")]
    VotesDontAddUp,

}

#[error_code]
pub enum CpiError {

    #[msg("The program ID for CPI was unexpected")]
    WrongProgramID,

}
