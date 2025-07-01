use crate::constants::DIV_BUFFER;

pub fn compute_returns(
    direction: bool,
    a: u64,
    b: u64,
    c: u64,
    vnf: u64,
    vna: u64,
    vu: u64,
) -> (u64, u64) {

    // Dirac Deltas based on direction (for/against)
    let delta_for: u128 = match direction {
        true => 1_u128,
        false => 0_u128,
    };
    let delta_against: u128 = 1_u128 - delta_for;

    // The maths *should* just work
    let against_for_ratio: u128 =    (DIV_BUFFER*(b as u128*(a as u128 + b as u128) + a as u128*c as u128)) / (a as u128*(a as u128 + b as u128) + b as u128*c as u128);
    let for_against_ratio: u128 =    (DIV_BUFFER*(a as u128*(a as u128 + b as u128) + b as u128*c as u128)) / (b as u128*(a as u128 + b as u128) + a as u128*c as u128);

    let underdog_for: u128 =       (DIV_BUFFER * b as u128*vu as u128) / (a as u128 + b as u128);
    let underdog_against: u128 =   (DIV_BUFFER * a as u128*vu as u128) / (a as u128 + b as u128);

    let bet_returned: u128 =    delta_for *     (vnf as u128 + (underdog_for / DIV_BUFFER)) +
                                delta_against * (vna as u128 + (underdog_against / DIV_BUFFER));
    let winnings_pre: u128 =    delta_for *      ((against_for_ratio * vnf as u128) + (against_for_ratio * underdog_for / DIV_BUFFER))      / DIV_BUFFER +
                                delta_against *  ((for_against_ratio * vna as u128) + (for_against_ratio * underdog_against / DIV_BUFFER))  / DIV_BUFFER;

    return (bet_returned.try_into().unwrap(), winnings_pre.try_into().unwrap())

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

pub fn vec_eq<T: std::cmp::Ord>(
    v1: &mut Vec<T>,
    v2: &mut Vec<T>,
) -> bool {

    v1.sort();
    v2.sort();

    v1 == v2

}

#[cfg(test)]
mod test_functions {
    use super::*;

    #[test]
    fn test_compute_returns() {

        let (bettor_tot_for_1, bettor_tot_against_1, bettor_tot_underdog_1) = (10_000_u64,  0_u64,      0_u64);
        let (bettor_tot_for_2, bettor_tot_against_2, bettor_tot_underdog_2) = (0_u64,       10_000_u64, 0_u64);
        let (bettor_tot_for_3, bettor_tot_against_3, bettor_tot_underdog_3) = (0_u64,       0_u64,      10_000_u64);

        let (escrow_tot_for_1, escrow_tot_against_1, escrow_tot_underdog_1) = (1_000_000_u64,   750_000_u64,    100_000_u64);
        let (escrow_tot_for_2, escrow_tot_against_2, escrow_tot_underdog_2) = (750_000_u64,     1_000_000_u64,  100_000_u64);
        let (escrow_tot_for_3, escrow_tot_against_3, escrow_tot_underdog_3) = (1_000_000_u64,   1_000_000_u64,  100_000_u64);
        let (escrow_tot_for_4, escrow_tot_against_4, escrow_tot_underdog_4) = (1_000_000_u64,   750_000_u64,    0_u64);
        let (escrow_tot_for_5, escrow_tot_against_5, escrow_tot_underdog_5) = (750_000_u64,     1_000_000_u64,  0_u64);
        let (escrow_tot_for_6, escrow_tot_against_6, escrow_tot_underdog_6) = (1_000_000_u64,   1_000_000_u64,  0_u64);


        // ----- BETTOR SCENARIO 1 -----

        let (bet_returned_true_11, winnings_pre_true_11) = compute_returns(
            true,
            escrow_tot_for_1,
            escrow_tot_against_1,
            escrow_tot_underdog_1,
            bettor_tot_for_1,
            bettor_tot_against_1,
            bettor_tot_underdog_1,
        );
        let (bet_returned_false_11, winnings_pre_false_11) = compute_returns(
            false,
            escrow_tot_for_1,
            escrow_tot_against_1,
            escrow_tot_underdog_1,
            bettor_tot_for_1,
            bettor_tot_against_1,
            bettor_tot_underdog_1,
        );
        assert_eq!(bet_returned_true_11, bettor_tot_for_1);
        assert_eq!(winnings_pre_true_11, 7_739); // 1042857.1 for, 807142.9 against --> 7739.7 winnings
        assert_eq!(bet_returned_false_11, 0_u64);
        assert_eq!(winnings_pre_false_11, 0_u64);

        let (bet_returned_true_21, winnings_pre_true_21) = compute_returns(
            true,
            escrow_tot_for_2,
            escrow_tot_against_2,
            escrow_tot_underdog_2,
            bettor_tot_for_1,
            bettor_tot_against_1,
            bettor_tot_underdog_1,
        );
        let (bet_returned_false_21, winnings_pre_false_21) = compute_returns(
            false,
            escrow_tot_for_2,
            escrow_tot_against_2,
            escrow_tot_underdog_2,
            bettor_tot_for_1,
            bettor_tot_against_1,
            bettor_tot_underdog_1,
        );
        assert_eq!(bet_returned_true_21, bettor_tot_for_1);
        assert_eq!(winnings_pre_true_21, 12_920); // 807142.9 for, 1042857.1 against --> 12920.4 winnings
        assert_eq!(bet_returned_false_21, 0_u64);
        assert_eq!(winnings_pre_false_21, 0_u64);

        let (bet_returned_true_31, winnings_pre_true_31) = compute_returns(
            true,
            escrow_tot_for_3,
            escrow_tot_against_3,
            escrow_tot_underdog_3,
            bettor_tot_for_1,
            bettor_tot_against_1,
            bettor_tot_underdog_1,
        );
        let (bet_returned_false_31, winnings_pre_false_31) = compute_returns(
            false,
            escrow_tot_for_3,
            escrow_tot_against_3,
            escrow_tot_underdog_3,
            bettor_tot_for_1,
            bettor_tot_against_1,
            bettor_tot_underdog_1,
        );
        assert_eq!(bet_returned_true_31, bettor_tot_for_1);
        assert_eq!(winnings_pre_true_31, bettor_tot_for_1); // Winnings same as initial bet
        assert_eq!(bet_returned_false_31, 0_u64);
        assert_eq!(winnings_pre_false_31, 0_u64);

        let (bet_returned_true_41, winnings_pre_true_41) = compute_returns(
            true,
            escrow_tot_for_4,
            escrow_tot_against_4,
            escrow_tot_underdog_4,
            bettor_tot_for_1,
            bettor_tot_against_1,
            bettor_tot_underdog_1,
        );
        let (bet_returned_false_41, winnings_pre_false_41) = compute_returns(
            false,
            escrow_tot_for_4,
            escrow_tot_against_4,
            escrow_tot_underdog_4,
            bettor_tot_for_1,
            bettor_tot_against_1,
            bettor_tot_underdog_1,
        );
        assert_eq!(bet_returned_true_41, bettor_tot_for_1);
        assert_eq!(winnings_pre_true_41, 7_500);
        assert_eq!(bet_returned_false_41, 0_u64);
        assert_eq!(winnings_pre_false_41, 0_u64);

        let (bet_returned_true_51, winnings_pre_true_51) = compute_returns(
            true,
            escrow_tot_for_5,
            escrow_tot_against_5,
            escrow_tot_underdog_5,
            bettor_tot_for_1,
            bettor_tot_against_1,
            bettor_tot_underdog_1,
        );
        let (bet_returned_false_51, winnings_pre_false_51) = compute_returns(
            false,
            escrow_tot_for_5,
            escrow_tot_against_5,
            escrow_tot_underdog_5,
            bettor_tot_for_1,
            bettor_tot_against_1,
            bettor_tot_underdog_1,
        );
        assert_eq!(bet_returned_true_51, bettor_tot_for_1);
        assert_eq!(winnings_pre_true_51, 13_333);
        assert_eq!(bet_returned_false_51, 0_u64);
        assert_eq!(winnings_pre_false_51, 0_u64);

        let (bet_returned_true_61, winnings_pre_true_61) = compute_returns(
            true,
            escrow_tot_for_6,
            escrow_tot_against_6,
            escrow_tot_underdog_6,
            bettor_tot_for_1,
            bettor_tot_against_1,
            bettor_tot_underdog_1,
        );
        let (bet_returned_false_61, winnings_pre_false_61) = compute_returns(
            false,
            escrow_tot_for_6,
            escrow_tot_against_6,
            escrow_tot_underdog_6,
            bettor_tot_for_1,
            bettor_tot_against_1,
            bettor_tot_underdog_1,
        );
        assert_eq!(bet_returned_true_61, bettor_tot_for_1);
        assert_eq!(winnings_pre_true_61, bettor_tot_for_1); // Winnings same as initial bet
        assert_eq!(bet_returned_false_61, 0_u64);
        assert_eq!(winnings_pre_false_61, 0_u64);


        // ----- BETTOR SCENARIO 2 -----

        let (bet_returned_true_12, winnings_pre_true_12) = compute_returns(
            true,
            escrow_tot_for_1,
            escrow_tot_against_1,
            escrow_tot_underdog_1,
            bettor_tot_for_2,
            bettor_tot_against_2,
            bettor_tot_underdog_2,
        );
        let (bet_returned_false_12, winnings_pre_false_12) = compute_returns(
            false,
            escrow_tot_for_1,
            escrow_tot_against_1,
            escrow_tot_underdog_1,
            bettor_tot_for_2,
            bettor_tot_against_2,
            bettor_tot_underdog_2,
        );
        assert_eq!(bet_returned_true_12, 0_u64);
        assert_eq!(winnings_pre_true_12, 0_u64);
        assert_eq!(bet_returned_false_12, bettor_tot_against_2);
        assert_eq!(winnings_pre_false_12, 12_920); // 1042857.1 for, 807142.9 against --> 12920.4 winnings

        let (bet_returned_true_22, winnings_pre_true_22) = compute_returns(
            true,
            escrow_tot_for_2,
            escrow_tot_against_2,
            escrow_tot_underdog_2,
            bettor_tot_for_2,
            bettor_tot_against_2,
            bettor_tot_underdog_2,
        );
        let (bet_returned_false_22, winnings_pre_false_22) = compute_returns(
            false,
            escrow_tot_for_2,
            escrow_tot_against_2,
            escrow_tot_underdog_2,
            bettor_tot_for_2,
            bettor_tot_against_2,
            bettor_tot_underdog_2,
        );
        assert_eq!(bet_returned_true_22, 0_u64);
        assert_eq!(winnings_pre_true_22, 0_u64);
        assert_eq!(bet_returned_false_22, bettor_tot_against_2);
        assert_eq!(winnings_pre_false_22, 7_739); // 807142.9 for, 1042857.1 against --> 7739.7 winnings

        let (bet_returned_true_32, winnings_pre_true_32) = compute_returns(
            true,
            escrow_tot_for_3,
            escrow_tot_against_3,
            escrow_tot_underdog_3,
            bettor_tot_for_2,
            bettor_tot_against_2,
            bettor_tot_underdog_2,
        );
        let (bet_returned_false_32, winnings_pre_false_32) = compute_returns(
            false,
            escrow_tot_for_3,
            escrow_tot_against_3,
            escrow_tot_underdog_3,
            bettor_tot_for_2,
            bettor_tot_against_2,
            bettor_tot_underdog_2,
        );
        assert_eq!(bet_returned_true_32, 0_u64);
        assert_eq!(winnings_pre_true_32, 0_u64);
        assert_eq!(bet_returned_false_32, bettor_tot_against_2);
        assert_eq!(winnings_pre_false_32, bettor_tot_against_2); // Winnings same as initial bet

        let (bet_returned_true_42, winnings_pre_true_42) = compute_returns(
            true,
            escrow_tot_for_4,
            escrow_tot_against_4,
            escrow_tot_underdog_4,
            bettor_tot_for_2,
            bettor_tot_against_2,
            bettor_tot_underdog_2,
        );
        let (bet_returned_false_42, winnings_pre_false_42) = compute_returns(
            false,
            escrow_tot_for_4,
            escrow_tot_against_4,
            escrow_tot_underdog_4,
            bettor_tot_for_2,
            bettor_tot_against_2,
            bettor_tot_underdog_2,
        );
        assert_eq!(bet_returned_true_42, 0_u64);
        assert_eq!(winnings_pre_true_42, 0_u64);
        assert_eq!(bet_returned_false_42, bettor_tot_against_2);
        assert_eq!(winnings_pre_false_42, 13_333);

        let (bet_returned_true_52, winnings_pre_true_52) = compute_returns(
            true,
            escrow_tot_for_5,
            escrow_tot_against_5,
            escrow_tot_underdog_5,
            bettor_tot_for_2,
            bettor_tot_against_2,
            bettor_tot_underdog_2,
        );
        let (bet_returned_false_52, winnings_pre_false_52) = compute_returns(
            false,
            escrow_tot_for_5,
            escrow_tot_against_5,
            escrow_tot_underdog_5,
            bettor_tot_for_2,
            bettor_tot_against_2,
            bettor_tot_underdog_2,
        );
        assert_eq!(bet_returned_true_52, 0_u64);
        assert_eq!(winnings_pre_true_52, 0_u64);
        assert_eq!(bet_returned_false_52, bettor_tot_against_2);
        assert_eq!(winnings_pre_false_52, 7_500);

        let (bet_returned_true_62, winnings_pre_true_62) = compute_returns(
            true,
            escrow_tot_for_6,
            escrow_tot_against_6,
            escrow_tot_underdog_6,
            bettor_tot_for_2,
            bettor_tot_against_2,
            bettor_tot_underdog_2,
        );
        let (bet_returned_false_62, winnings_pre_false_62) = compute_returns(
            false,
            escrow_tot_for_6,
            escrow_tot_against_6,
            escrow_tot_underdog_6,
            bettor_tot_for_2,
            bettor_tot_against_2,
            bettor_tot_underdog_2,
        );
        assert_eq!(bet_returned_true_62, 0_u64);
        assert_eq!(winnings_pre_true_62, 0_u64);
        assert_eq!(bet_returned_false_62, bettor_tot_against_2);
        assert_eq!(winnings_pre_false_62, bettor_tot_against_2); // Winnings same as initial bet
        

        // ----- BETTOR SCENARIO 3 -----

        let (bet_returned_true_13, winnings_pre_true_13) = compute_returns(
            true,
            escrow_tot_for_1,
            escrow_tot_against_1,
            escrow_tot_underdog_1,
            bettor_tot_for_3,
            bettor_tot_against_3,
            bettor_tot_underdog_3,
        );
        let (bet_returned_false_13, winnings_pre_false_13) = compute_returns(
            false,
            escrow_tot_for_1,
            escrow_tot_against_1,
            escrow_tot_underdog_1,
            bettor_tot_for_3,
            bettor_tot_against_3,
            bettor_tot_underdog_3,
        );
        // 1042857.143 tot for, 807142.8571 tot against
        assert_eq!(bet_returned_true_13, 4_285);    // --> 4285.714286 back
        assert_eq!(winnings_pre_true_13, 3_317);    // --> 3317.02544 winnings
        assert_eq!(bet_returned_false_13, 5_714);   // --> 5714.285714 back
        assert_eq!(winnings_pre_false_13, 7_383);   // --> 7383.05942 winnings

        let (bet_returned_true_23, winnings_pre_true_23) = compute_returns(
            true,
            escrow_tot_for_2,
            escrow_tot_against_2,
            escrow_tot_underdog_2,
            bettor_tot_for_3,
            bettor_tot_against_3,
            bettor_tot_underdog_3,
        );
        let (bet_returned_false_23, winnings_pre_false_23) = compute_returns(
            false,
            escrow_tot_for_2,
            escrow_tot_against_2,
            escrow_tot_underdog_2,
            bettor_tot_for_3,
            bettor_tot_against_3,
            bettor_tot_underdog_3,
        );
        // 807142.8571 tot for, 1042857.143 tot against
        assert_eq!(bet_returned_true_23, 5_714);    // --> 5714.285714 back
        assert_eq!(winnings_pre_true_23, 7_383);    // --> 7383.05942 winnings
        assert_eq!(bet_returned_false_23, 4_285);   // --> 4285.714286 back
        assert_eq!(winnings_pre_false_23, 3_317);   // --> 3317.02544 winnings

        let (bet_returned_true_33, winnings_pre_true_33) = compute_returns(
            true,
            escrow_tot_for_3,
            escrow_tot_against_3,
            escrow_tot_underdog_3,
            bettor_tot_for_3,
            bettor_tot_against_3,
            bettor_tot_underdog_3,
        );
        let (bet_returned_false_33, winnings_pre_false_33) = compute_returns(
            false,
            escrow_tot_for_3,
            escrow_tot_against_3,
            escrow_tot_underdog_3,
            bettor_tot_for_3,
            bettor_tot_against_3,
            bettor_tot_underdog_3,
        );
        assert_eq!(bet_returned_true_33, bettor_tot_underdog_3/2);
        assert_eq!(winnings_pre_true_33, bettor_tot_underdog_3/2);
        assert_eq!(bet_returned_false_33, bettor_tot_underdog_3/2);
        assert_eq!(winnings_pre_false_33, bettor_tot_underdog_3/2);

        // escrow scenarios 4-6 don't make sense for bettor scenario 3 because there must be at least some underdog bets
        
    }

    #[test]
    fn test_calc_winnings_from_votes() {

        let amount: u64 = 1_000_000_u64;

        let poll_direction_1: bool = true;
        let voter_direction_1: bool = true;
        let amount_1: u64 = calc_winnings_from_votes(
            poll_direction_1,
            voter_direction_1,
            amount,
        );
        assert_eq!(amount_1, amount);

        let poll_direction_2: bool = true;
        let voter_direction_2: bool = false;
        let amount_2: u64 = calc_winnings_from_votes(
            poll_direction_2,
            voter_direction_2,
            amount,
        );
        assert_eq!(amount_2, 0_u64);

        let poll_direction_3: bool = false;
        let voter_direction_3: bool = true;
        let amount_3: u64 = calc_winnings_from_votes(
            poll_direction_3,
            voter_direction_3,
            amount,
        );
        assert_eq!(amount_3, 0_u64);

        let poll_direction_4: bool = false;
        let voter_direction_4: bool = false;
        let amount_4: u64 = calc_winnings_from_votes(
            poll_direction_4,
            voter_direction_4,
            amount,
        );
        assert_eq!(amount_4, amount);

    }

    #[test]
    fn test_vec_eq() {
        use anchor_lang::prelude::Pubkey;

        let vec_1: &mut Vec<u32> = &mut Vec::from([1243, 542, 7624, 3000, 54]);
        let vec_2: &mut Vec<u32> = &mut Vec::from([3000, 1243, 54, 542, 7644]);

        let result_1: bool = vec_eq(vec_1, vec_2);

        assert!(!result_1);


        let vec_1: &mut Vec<u32> = &mut Vec::from([1243, 542, 7644, 3000, 54]);
        let vec_2: &mut Vec<u32> = &mut Vec::from([3000, 1243, 54, 542, 7644]);

        let result_2: bool = vec_eq(vec_1, vec_2);

        assert!(result_2);


        let vec_1: &mut Vec<u32> = &mut Vec::from([1243, 542, 7644, 3000, 54, 2]);
        let vec_2: &mut Vec<u32> = &mut Vec::from([3000, 1243, 54, 542, 7644]);

        let result_3: bool = vec_eq(vec_1, vec_2);

        assert!(!result_3);


        let vec_1: &mut Vec<u32> = &mut Vec::from([1243, 542, 7644, 3000, 54]);
        let vec_2: &mut Vec<u32> = &mut Vec::from([3000, 1243, 54, 542, 7644, 2]);

        let result_4: bool = vec_eq(vec_1, vec_2);

        assert!(!result_4);


        // --------------------------------------------------------------------
        // --------------------------------------------------------------------

        let pk_1: Pubkey = Pubkey::new_unique();
        let pk_2: Pubkey = Pubkey::new_unique();
        let pk_3: Pubkey = Pubkey::new_unique();
        let pk_4: Pubkey = Pubkey::new_unique();
        let pk_5: Pubkey = Pubkey::new_unique();
        let pk_6: Pubkey = Pubkey::new_unique();

        let vec_1: &mut Vec<Pubkey> = &mut Vec::from([pk_5, pk_4, pk_6, pk_3, pk_2]);
        let vec_2: &mut Vec<Pubkey> = &mut Vec::from([pk_1, pk_2, pk_3, pk_4, pk_5]);

        let result_5: bool = vec_eq(vec_1, vec_2);

        assert!(!result_5);


        let vec_1: &mut Vec<Pubkey> = &mut Vec::from([pk_5, pk_4, pk_1, pk_3, pk_2]);
        let vec_2: &mut Vec<Pubkey> = &mut Vec::from([pk_1, pk_2, pk_3, pk_4, pk_5]);

        let result_6: bool = vec_eq(vec_1, vec_2);

        assert!(result_6);


        let vec_1: &mut Vec<Pubkey> = &mut Vec::from([pk_5, pk_4, pk_1, pk_3, pk_2, pk_6]);
        let vec_2: &mut Vec<Pubkey> = &mut Vec::from([pk_1, pk_2, pk_3, pk_4, pk_5]);

        let result_7: bool = vec_eq(vec_1, vec_2);

        assert!(!result_7);


        let vec_1: &mut Vec<Pubkey> = &mut Vec::from([pk_5, pk_4, pk_1, pk_3, pk_2]);
        let vec_2: &mut Vec<Pubkey> = &mut Vec::from([pk_1, pk_2, pk_3, pk_4, pk_5, pk_6]);

        let result_8: bool = vec_eq(vec_1, vec_2);

        assert!(!result_8);

    }

}
