use anchor_lang::prelude::*;

use crate::constants::DIV_BUFFER;

pub fn compute_returns(
    direction: bool,
    escrow_tot_for: u64,
    escrow_tot_against: u64,
    escrow_tot_underdog: u64,
    bettor_tot_for: u64,
    bettor_tot_against: u64,
    bettor_tot_underdog: u64,
) -> (u64, u64) {
    let for_multiplier: u64 = match direction {
        true => 1_u64,
        false => 0_u64,
    };
    let against_multiplier: u64 = 1_u64 - for_multiplier;

    // Escrow totals for and against with appropriate shares from underdog bets
    let final_tot_for: u64 = (DIV_BUFFER*escrow_tot_for + (DIV_BUFFER*escrow_tot_underdog*escrow_tot_against)/(escrow_tot_for + escrow_tot_against))/DIV_BUFFER;
    let final_tot_against: u64 = (DIV_BUFFER*escrow_tot_against + (DIV_BUFFER*escrow_tot_underdog*escrow_tot_for)/(escrow_tot_for + escrow_tot_against))/DIV_BUFFER;

    // Final bets for and against from user underdog bets
    let underdog_for: u64 = ((DIV_BUFFER*bettor_tot_underdog*final_tot_against)/(final_tot_against + final_tot_against))/DIV_BUFFER;
    let underdog_against: u64 = ((DIV_BUFFER*bettor_tot_underdog*final_tot_for)/(final_tot_against + final_tot_against))/DIV_BUFFER;

    // Winnings for and against from user normal bets
    let winnings_for: u64 = ((DIV_BUFFER*final_tot_against*bettor_tot_for)/final_tot_for)/DIV_BUFFER;
    let winnings_against: u64 = ((DIV_BUFFER*final_tot_for*bettor_tot_against)/final_tot_against)/DIV_BUFFER;

    // Winnings for and against from user underdog bets
    let underdog_winnings_for: u64 = ((DIV_BUFFER*final_tot_against*underdog_for)/final_tot_for)/DIV_BUFFER;
    let underdog_winnings_against: u64 = ((DIV_BUFFER*final_tot_for*underdog_against)/final_tot_against)/DIV_BUFFER;

    let bet_returned: u64 = for_multiplier*(bettor_tot_for + underdog_for)
                            + against_multiplier*(bettor_tot_against + underdog_against);
    let winnings_pre: u64 = for_multiplier*(winnings_for + underdog_winnings_for)
                            + against_multiplier*(winnings_against + underdog_winnings_against);

    return (bet_returned, winnings_pre)

}

pub fn calc_winnings_from_votes(
    poll_direction: bool,
    voter_direction: bool,
    amount: u64
) -> u64 {
    match !(poll_direction ^ voter_direction) {
        true => amount,
        false => 0,
    }
}

pub fn vec_eq(
    v1: &mut Vec<Pubkey>,
    v2: &mut Vec<Pubkey>,
) -> bool {

    v1.sort();
    v2.sort();

    v1 == v2

}
