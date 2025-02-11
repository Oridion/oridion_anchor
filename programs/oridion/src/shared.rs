use super::*;

pub fn hop_deposit(deposit_account: &mut Account<Deposit>){
    let clock: Clock = Clock::get().unwrap();
    let now = clock.unix_timestamp;
    let withdraw_time = deposit_account.withdraw_at;

    deposit_account.hops += 1;
    deposit_account.last_process = 1; // Last action -> (1 = hop)
    deposit_account.last_process_at = now;

    //Manual, Instant, Delay
    //Since we will be able to trigger hop manually through lambda function,
    //We update delay type deposits only
    if deposit_account.mode == 1 {
        //Depending on the withdrawal timestamp, set the next process and hop process timestamp
        if (now + 180) > withdraw_time {
            //Withdraw is the next action.
            deposit_account.next_process = 1; //1 = withdraw
            deposit_account.next_process_at = withdraw_time;
        } else {
            //Set the next hop processing timestamp
            deposit_account.next_process = 0; //0 = hop
            deposit_account.next_process_at = now + 180;
        }
    }
}

pub fn get_planet_program_address(planet_name: &String, program_id: &Pubkey) -> Pubkey {
    let(pk, _pda_bump) = Pubkey::find_program_address(&[
        PLANET_PDA_SEED_PRE,
        planet_name.as_ref(),
        PLANET_PDA_SEED_POST
    ], program_id);
    pk
}

pub fn get_random_percent() -> f32 {
    let clock: Clock = Clock::get().unwrap();
    // First get the percent to split deposit between two stars.
    let clock_time_str: String = clock.unix_timestamp.to_string();
    let percent: &str = {
        let split_pos: usize = clock_time_str.char_indices().nth_back(1).unwrap().0;
        &clock_time_str[split_pos..]
    };

    //msg!("Lamports splitting: {}", deposit.to_string());
    let percent: f32 = percent.parse::<f32>().unwrap();
    if percent < 10f32 {
        return 10f32
    } else if percent > 90f32 {
        return 90f32
    } else {
        return percent
    }
}
