use anchor_lang::prelude::*;
use super::*;

///------------------------------------------------------------//
/// COMET PDA
/// Created by users.
/// Creator must submit a unique 4-digit code for each comet created.
/// User creates a comet which will transcend through stars for a various amount of hops
/// before ending at a destination address.
/// - id
/// - creator
/// - created
/// - pda
/// - bump
/// - deposit
/// - destination
/// - hops
/// - hops_completed
/// - last_updated
///
/// useful for later
/// new anchor.BN(0).toArrayLike(Buffer)
/// https://stackoverflow.com/questions/71807112/how-to-derive-pdas-with-multiple-seeds-in-anchor-rust
///
///------------------------------------------------------------//
#[derive(Accounts)]
pub struct CreateDeposit<'info> {
    #[account(
        init,
        payer = creator,
        space = DISCRIMINATOR_LENGTH + Deposit::INIT_SPACE,
        seeds = [
            DEPOSIT_PDA_SEED_PRE,
            creator.key().as_ref(),
            DEPOSIT_PDA_SEED_POST
        ],
        bump
    )]
    pub deposit: Account<'info,Deposit>,
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut)]
    pub universe: Account<'info,Universe>,
    #[account(mut)]
    pub planet: Account<'info,Planet>,
    pub destination: SystemAccount<'info>,
    pub system_program: Program<'info,System>,
    pub rent: Sysvar<'info, Rent>,
}


// Create new deposit data account
// Todo: Missing activity and processing
// activity - '[{"action": "D", "to": "BABALO", "time": "1711686392", "signature": "2Cems9LQFpDJaE21m4Hfiyr8BPdDmeeMVQJGpuFAU4XoSXGqnTep2AJiqyQrJPbDSBwDVT5ZQ9bpqi3HRXZ9vy4x"}, {"action": "HP", "to": "VREDIA", "time": 1711686438, "signature": "5Zwyi1sDZR5nPgYf4G7wBaZeYqoHCrB84d15B3rjuTTGV3mtdLfHufV6vvFusWqrEN6UtdaLuvGiphqqSLGnESzW"}]',
// processing - removed, not sure if we are going to need this.
#[account]
#[derive(InitSpace)]
pub struct Deposit {
    pub mode: u8, //1 Delay, 2 Instant, 3 Manual
    pub next_process_at: i64, //Next process timestamp
    pub next_process: u8, // (0 'hop'  1 'withdraw')
    pub last_process: u8, //('0 - deposit','1 - hop')
    #[max_len(10)]
    pub location: String, // Current planet location
    pub delay: u32, // Set delay in seconds
    pub hops: u16, //Amount of hops
    pub withdraw_at: i64, //Set withdraw timestamp
    pub created_at: i64, //Deposit started
    pub last_process_at: i64, //Last updated timestamp
    pub lamports: u64, //Lamports deposited
    pub destination: Pubkey //Destination wallet address
}

//Constrain = The "creator public key" being passed during this update
//must match the "creator" field already set in the data of the comet account when initialized
//Try to use this for HOP and DESTINATION RETRIEVAL FUNDS
//Hope to planet will always go from planet to planet
#[derive(Accounts)]
pub struct PlanetHop<'info> {
    #[account(mut)]
    pub deposit: Account<'info, Deposit>,
    #[account(mut)]
    pub to_planet: Account<'info,Planet>,
    #[account(mut)]
    pub from_planet: Account<'info,Planet>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>
}

#[derive(Accounts)]
pub struct WithdrawAccounts<'info> {
    #[account(mut, close = manager)]
    pub deposit: Account<'info, Deposit>,
    #[account(mut)]
    pub from_planet: Account<'info,Planet>,
    #[account(mut, address = deposit.destination)]
    pub destination: SystemAccount<'info>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>
}



// // V2 - 5 PLANET SPREAD HOP
// #[derive(Accounts)]
// pub struct PlanetShotGunStart<'info> {
//     #[account(mut)]
//     pub from_planet: Account<'info,Planet>,
//     #[account(mut)]
//     pub to_planet_1: Account<'info,Planet>,
//     #[account(mut)]
//     pub to_planet_2: Account<'info,Planet>,
//     #[account(mut)]
//     pub to_planet_3: Account<'info,Planet>,
//     #[account(mut)]
//     pub to_planet_4: Account<'info,Planet>,
//     #[account(mut)]
//     pub to_planet_5: Account<'info,Planet>,
//     #[account(mut, address = MANAGER_PUBKEY)]
//     pub manager: Signer<'info>
// }
//
//
// //Return from stars to destination planet
// #[derive(Accounts)]
// pub struct PlanetShotGunEnd<'info> {
//     #[account(mut)]
//     pub from_planet_1: Account<'info,Planet>,
//     #[account(mut)]
//     pub from_planet_2: Account<'info,Planet>,
//     #[account(mut)]
//     pub from_planet_3: Account<'info,Planet>,
//     #[account(mut)]
//     pub from_planet_4: Account<'info,Planet>,
//     #[account(mut)]
//     pub from_planet_5: Account<'info,Planet>,
//     #[account(mut)]
//     pub to_planet: Account<'info,Planet>,
//     #[account(mut, address = MANAGER_PUBKEY)]
//     pub manager: Signer<'info>
// }



//Star hop from Planet to Split stars
#[derive(Accounts)]
#[instruction(star_one_id: String, star_two_id: String)]
pub struct StarHopTwoStart<'info> {
    #[account(mut)]
    pub deposit: Account<'info, Deposit>,
    #[account(mut)]
    pub from_planet: Account<'info,Planet>,
    #[account(init, payer = manager, space = DISCRIMINATOR_LENGTH + Star::INIT_SPACE,
        seeds = [
            STAR_SEED_PRE,
            star_one_id.as_ref(),
            STAR_SEED_POST
        ],
        bump
    )]
    pub star_one: Account<'info, Star>,
    #[account(init, payer = manager, space = DISCRIMINATOR_LENGTH + Star::INIT_SPACE,
        seeds = [
            STAR_SEED_PRE,
            star_two_id.as_ref(),
            STAR_SEED_POST
        ],
        bump
    )]
    pub star_two: Account<'info, Star>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

//Return from stars to destination planet
#[derive(Accounts)]
pub struct StarHopTwoEnd<'info> {
    #[account(mut)]
    pub deposit: Account<'info, Deposit>,
    #[account(mut)]
    pub to_planet: Account<'info,Planet>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_one.manager)]
    pub star_one: Account<'info, Star>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_two.manager)]
    pub star_two: Account<'info, Star>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>
}


#[derive(Accounts)]
#[instruction(star_one_id: String, star_two_id: String, star_three_id: String )]
pub struct StarHopThreeStart<'info> {
    #[account(mut)]
    pub deposit: Account<'info, Deposit>,
    #[account(mut)]
    pub from_planet: Account<'info,Planet>,
    #[account(init, payer = manager, space = DISCRIMINATOR_LENGTH + Star::INIT_SPACE,
        seeds = [
            STAR_SEED_PRE,
            star_one_id.as_ref(),
            STAR_SEED_POST
        ],
        bump
    )]
    pub star_one: Account<'info, Star>,

    #[account(init, payer = manager, space = DISCRIMINATOR_LENGTH + Star::INIT_SPACE,
        seeds = [
            STAR_SEED_PRE,
            star_two_id.as_ref(),
            STAR_SEED_POST
        ],
        bump
    )]
    pub star_two: Account<'info, Star>,

    #[account(init, payer = manager, space = DISCRIMINATOR_LENGTH + Star::INIT_SPACE,
        seeds = [
            STAR_SEED_PRE,
            star_three_id.as_ref(),
            STAR_SEED_POST
        ],
        bump
    )]
    pub star_three: Account<'info, Star>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StarHopThreeEnd<'info> {
    #[account(mut)]
    pub deposit: Account<'info, Deposit>,
    #[account(mut)]
    pub to_planet: Account<'info,Planet>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_one.manager)]
    pub star_one: Account<'info, Star>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_two.manager)]
    pub star_two: Account<'info, Star>,
    #[account(mut, close = manager, has_one = manager, constraint = manager.key == &star_three.manager)]
    pub star_three: Account<'info, Star>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub manager: Signer<'info>
}

#[account]
#[derive(InitSpace)]
pub struct Star {
    pub amount: u64,
    pub manager: Pubkey
}