mod variables;
mod accounts_universe;
mod accounts_comet;
mod accounts_planet;
mod shared;
mod errors;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program;
use solana_security_txt::security_txt;
use variables::*;
use shared::*;
use accounts_universe::*;
use accounts_comet::*;
use accounts_planet::*;
use errors::*;

declare_id!("33J2bC6ZYvg8Y77MWULZEWpWRkxCoM2aziZ5f67dwBXi");

security_txt! {
    name: "Oridion",
    project_url: "https://oridion.xyz",
    source_code: "https://github.com/Oridion/oridion_anchor",
    preferred_languages: "en",
    contacts: "twitter:@OridionGalaxy,email:oridion.xyz@gmail.com",
    policy: "https://oridion.xyz/privacy-policy"
}

#[program]
pub mod oridion {
    use solana_program::instruction::Instruction;
    use super::*;

    ///-------------------------------------------------------------------///
    /// UNIVERSE
    ///-------------------------------------------------------------------///
    pub fn bang(ctx: Context<BigBang>) -> Result<()> {
        let clock: Clock = Clock::get().unwrap();
        let universe: &mut Account<Universe> = &mut ctx.accounts.universe;
        universe.bp = ctx.bumps.universe; // store bump seed in `Counter` account
        universe.p = Vec::<String>::new();
        universe.st = clock.unix_timestamp;
        universe.up = clock.unix_timestamp; //must set this here as well for random comet id
        universe.cfe = 0; //Comet fee Lamports (0) - Starts at 0 cents (when solana is $100 per 1)
        universe.hpfe = 1000000; //Hop planet lamports (1000000)
        universe.hsfe2 = 2000000; //Hop star lamports (2000000)
        universe.hsfe3 = 3000000; //Hop star lamports (3000000)
        universe.wfe = 0; //Withdraw Lamports (0) - no withdraw fee - Starts at .0 cents (when solana is $100 per 1)
        let(pda, _bump_seed) = Pubkey::find_program_address(&[UNIVERSE_PDA_SEED], &ctx.program_id);
        universe.pda = pda;
        Ok(())
    }

    /// -------------------------------------------------------------------///
    /// UPDATE FEE
    ///-------------------------------------------------------------------///
    pub fn update_fee(ctx: Context<UpdateUniverseFee>, comet_fee: u32, hop_planet_fee: u32, hop_star_fee2: u32, hop_star_fee3: u32, withdraw_fee: u32) -> Result<()> {
        let clock: Clock = Clock::get().unwrap();
        let universe: &mut Account<Universe> = &mut ctx.accounts.universe;
        universe.up = clock.unix_timestamp; //must set this here as well for random comet id
        universe.cfe = comet_fee as u64; //Lamports
        universe.hpfe = hop_planet_fee as u64; //Lamports
        universe.hsfe2 = hop_star_fee2 as u64; //Lamports
        universe.hsfe3 = hop_star_fee3 as u64; //Lamports
        universe.wfe = withdraw_fee as u64; //Lamports
        Ok(())
    }


    ///-------------------------------------------------------------------///
    /// CREATE PLANET
    ///-------------------------------------------------------------------///
    pub fn create_planet(ctx: Context<CreatePlanet>, name: String) -> Result<()> {
        let clock: Clock = Clock::get().unwrap();
        let planet: &mut Account<Planet> = &mut ctx.accounts.planet;
        let universe: &mut Account<Universe> = &mut ctx.accounts.universe;

        //Planet name too long
        require!(name.len() <= MAX_PLANET_TITLE_LENGTH, OridionError::PlanetNameTooLong);

        planet.name = name.clone();
        planet.created = clock.unix_timestamp;
        planet.bump = ctx.bumps.planet;
        planet.pda = get_planet_program_address(&name,&ctx.program_id);
        planet.visits = 0;
        universe.p.push(name.clone());
        Ok(())
    }

    ///-------------------------------------------------------------------///
    /// DELETE PLANET
    ///-------------------------------------------------------------------///
    pub fn delete_planet(ctx: Context<DeletePlanet>) -> Result<()> {
        let planet_lamports = ctx.accounts.planet.get_lamports();

        //ERROR! PLANET HAS SOL! (1593840 == empty planet) 1593840
        require!(planet_lamports <= 1593840, OridionError::PlanetDeleteHasFundsError);

        //Remove planet from universe list
        let universe: &mut Account<Universe> = &mut ctx.accounts.universe;
        let planet: &mut Account<Planet> = &mut ctx.accounts.planet;
        universe.p.retain(|x| x != &planet.name);
        //msg!("== PLANET {} DELETED ==", planet.name.to_string());
        Ok(())
    }

    ///-------------------------------------------------------------------///
    /// CREATE DEPOSIT
    /// Creates user's deposit and handles transfer from galaxy to planet. 
    /// - Signed by user
    /// - Deposit (In lamports)
    /// - Occurs before creating deposit account.
    ///-------------------------------------------------------------------///
    pub fn create_deposit(ctx: Context<CreateDeposit>,deposit_lamports: u64, mode: u8, delay: u32, withdraw_at: i64 ) -> Result<()> {

        let clock: Clock = Clock::get().unwrap();
        let now = clock.unix_timestamp;
        //msg!("Now Timestamp: {}", now.to_string());

        // CREATE DEPOSIT ACCOUNT
        let deposit_account: &mut Account<Deposit> = &mut ctx.accounts.deposit;
        let destination: &AccountInfo = ctx.accounts.destination.as_ref();

        deposit_account.created_at = now;
        deposit_account.last_process_at = now;
        deposit_account.hops = 2;
        deposit_account.lamports = deposit_lamports;
        deposit_account.mode = mode;
        deposit_account.last_process = 0; //0 = deposit
        deposit_account.delay = delay;
        deposit_account.withdraw_at = withdraw_at;
        deposit_account.destination = *destination.key; //Destination must be set.

        //Depending on the withdrawal timestamp, set the next process and hop process timestamp
        if (now + 180) > withdraw_at {
            //Withdraw is the next action.
            deposit_account.next_process = 1; //1 = withdraw
            deposit_account.next_process_at = withdraw_at;
        } else {
            //Set the next hop processing timestamp
            deposit_account.next_process = 0; //0 = hop
            deposit_account.next_process_at = now + 180;
        }


        // INCREMENT PLANET VISIT
        let planet: &mut Account<Planet> = &mut ctx.accounts.planet;
        planet.visits += 1;


        //Set planet name to deposit
        deposit_account.location = planet.name.clone();


        // TRANSACTION SETUP
        let creator_account: &Signer = &ctx.accounts.creator;
        let universe_account: &Account<Universe> = &ctx.accounts.universe;

        // DEPOSIT TRANSFER
        let transfer_instruction: Instruction = system_instruction::transfer(creator_account.key, universe_account.to_account_info().key, deposit_lamports);
        solana_program::program::invoke_signed(
            &transfer_instruction,
            &[
                creator_account.to_account_info(),
                universe_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;

        // TRANSACTION - From galaxy to planet
        //msg!("Hopping from galaxy to {}", planet_account.name);
        ctx.accounts.planet.add_lamports(deposit_lamports)?;
        ctx.accounts.universe.sub_lamports(deposit_lamports)?;
        Ok(())
    }


    ///-------------------------------------------------------------------///
    /// HOP FROM Planet to Planet
    /// -------------------------------------------------------------------///
    pub fn planet_hop(ctx: Context<PlanetHop>) -> Result<()>{

        //Deposit account for updating location
        let deposit: &mut Account<Deposit> = &mut ctx.accounts.deposit;
        let from: &mut Account<Planet> = &mut ctx.accounts.from_planet;
        let to: &mut Account<Planet> = &mut ctx.accounts.to_planet;

        // IMPORTANT VALIDATION: TO AND FROM CANNOT BE THE SAME
        require!(from.name != to.name, OridionError::HopErrorToAndFromAreSame);

        // Update deposit with new data
        deposit.location = to.name.clone();
        hop_deposit(deposit);

        //Increment visits
        to.visits += 1;

        // TRANSACTION: Move funds from planet to planet
        ctx.accounts.to_planet.add_lamports(deposit.lamports)?;
        ctx.accounts.from_planet.sub_lamports(deposit.lamports)?;

        Ok(())
    }

    /// WITHDRAW COMET FUNDS TO FINAL DESTINATION.
    pub fn withdraw(ctx: Context<WithdrawAccounts>) -> Result<()> {

        //Get deposit account to determine how much to withdraw
        let deposit: &mut Account<Deposit> = &mut ctx.accounts.deposit;
        let withdraw_lamports = deposit.lamports.clone();

        let from: &mut Account<Planet> = &mut ctx.accounts.from_planet;

        //VALIDATION 
        let current_from_lamports_balance: u64 = from.get_lamports();
        require!(current_from_lamports_balance > withdraw_lamports, OridionError::PlanetNotEnoughFundsError);

        // TRANSACTION - Transfer to destination
        ctx.accounts.destination.add_lamports(withdraw_lamports)?;
        ctx.accounts.from_planet.sub_lamports(withdraw_lamports)?;

        //Remove remaining lamports from deposit
        let deposit_remaining_lamports = ctx.accounts.deposit.get_lamports();
        ctx.accounts.deposit.sub_lamports(deposit_remaining_lamports)?;
        ctx.accounts.manager.add_lamports(deposit_remaining_lamports)?;

        Ok(())
    }


    ///-------------------------------------------------------------------///
    /// STAR HOP SECTION
    ///-------------------------------------------------------------------///
     pub fn star_hop_two_start(ctx: Context<StarHopTwoStart>, star_one :String , star_two: String) -> Result<()>{

        // Deposit account
        let deposit: &mut Account<Deposit> = &mut ctx.accounts.deposit;
        let manager: &Signer = &ctx.accounts.manager;
        let star1: &mut Account<Star> = &mut ctx.accounts.star_one;
        let star2: &mut Account<Star> = &mut ctx.accounts.star_two;

        // IMPORTANT VALIDATION: STAR ONE AND TWO CANNOT BE THE SAME
        require!(star_one != star_two, OridionError::HopErrorStarsMustBeUnique);

        //msg!("Validation successful");
        let percent: f32 = get_random_percent();
        let star_one_amount: u64 = ((percent / 100f32) * deposit.lamports as f32) as u64;
        let star_two_amount: u64 = deposit.lamports - star_one_amount;
        //msg!("Hopping to star 1: {}", star_one_amount.to_string());
        //msg!("Hopping to Star 2: {}", star_two_amount.to_string());

        //Make sure the addition of split amounts are equal to deposit
        require!(star_one_amount + star_two_amount == deposit.lamports, OridionError::StarHopCalculationError);

        //Set amounts to accounts
        star1.amount = star_one_amount.clone();
        star2.amount = star_two_amount.clone();
        star1.manager = *manager.key;
        star2.manager = *manager.key;

        // TRANSACTION
        ctx.accounts.star_one.add_lamports(star_one_amount)?;
        ctx.accounts.star_two.add_lamports(star_two_amount)?;
        ctx.accounts.from_planet.sub_lamports(deposit.lamports)?;

        Ok(())
    }


    ///-------------------------------------------------------------------///
    /// STAR HOP SECTION
    ///-------------------------------------------------------------------///
    pub fn star_hop_two_end(ctx: Context<StarHopTwoEnd>) -> Result<()>{

        let deposit: &mut Account<Deposit> = &mut ctx.accounts.deposit;

        let to: &mut Account<Planet> = &mut ctx.accounts.to_planet;
        let star1: &mut Account<Star> = &mut ctx.accounts.star_one;
        let star2: &mut Account<Star> = &mut ctx.accounts.star_two;
        let star_one_amount: u64 = star1.amount.clone();
        let star_two_amount: u64 = star2.amount.clone();

        require!(star_one_amount + star_two_amount == deposit.lamports, OridionError::StarHopCalculationError);

        // Update deposit with new data
        deposit.location = to.name.clone();
        hop_deposit(deposit);

         //Clear our star amount
        star1.amount = 0;
        star2.amount = 0;
     
        //Increment planet visit
        to.visits += 1;
        
        // TRANSACTIONS
        // Transaction from stars one and two to destination planet       
        let total_lamports: u64 = star_one_amount + star_two_amount;
        ctx.accounts.star_one.sub_lamports(star_one_amount)?;
        ctx.accounts.star_two.sub_lamports(star_two_amount)?;
        ctx.accounts.to_planet.add_lamports(total_lamports)?;
        //msg!("Hop from two stars to {} complete", to_planet_name);     
        //msg!("Hop from two stars to planet complete");     

        //Transfer out remaining lamports
        let star_one_remaining_lamports = ctx.accounts.star_one.get_lamports();
        let star_two_remaining_lamports = ctx.accounts.star_two.get_lamports();
        ctx.accounts.manager.add_lamports(star_one_remaining_lamports)?;

        ctx.accounts.star_one.sub_lamports(star_one_remaining_lamports)?;
        ctx.accounts.star_two.sub_lamports(star_two_remaining_lamports)?;
        ctx.accounts.manager.add_lamports(star_two_remaining_lamports)?;

        Ok(())
    }




    
    /// STAR HOP THREE
    pub fn star_hop_three_start(ctx: Context<StarHopThreeStart>, star_one :String , star_two: String, star_three: String) -> Result<()>{

        // Deposit account
        let deposit: &mut Account<Deposit> = &mut ctx.accounts.deposit;

        //let clock: Clock = Clock::get().unwrap();
        let manager: &Signer = &ctx.accounts.manager;

        let star1: &mut Account<Star> = &mut ctx.accounts.star_one;
        let star2: &mut Account<Star> = &mut ctx.accounts.star_two;
        let star3: &mut Account<Star> = &mut ctx.accounts.star_three;

        // IMPORTANT VALIDATION: STAR ONE AND TWO CANNOT BE THE SAME
        require!(star_one != star_two && star_two != star_three && star_one != star_three, OridionError::HopErrorStarsMustBeUnique);

        let first_split_percent: f32 = get_random_percent();
        let second_split_percent: f32 = 100f32 - first_split_percent;
        //msg!("Split: {}% / {}%", first_split_percent.to_string(), second_split_percent.to_string());
        
        //Determine side amounts here
        let side_one_amount: u64 = ((first_split_percent / 100f32) * deposit.lamports as f32) as u64;
        let side_two_amount: u64 = deposit.lamports - side_one_amount;

        let (star_one_amount, star_two_amount, star_three_amount) = if side_one_amount > side_two_amount {
            //Side one is larger so we split side one.
            //Side two is set as star 2
            //We further split side one to create star one and star three
            let one: u64 = ((second_split_percent / 100f32) * side_one_amount as f32) as u64;
            let three: u64 = side_one_amount - one;
            //msg!("Third split on first side");
            (one,side_two_amount,three)
        } else {
            //Side two is larger so we split side two.
            //Side one is set as star one
            //We further split side two to create side two and side three
            let two: u64 = ((second_split_percent / 100f32) * side_two_amount as f32) as u64;
            let three: u64 = side_two_amount - two;
            //msg!("Third split on second side");
            (side_one_amount,two,three)
        };

        require!(star_one_amount + star_two_amount + star_three_amount == deposit.lamports, OridionError::StarHopCalculationError);

        star1.amount = star_one_amount;
        star2.amount = star_two_amount;
        star3.amount = star_three_amount;
        star1.manager = *manager.key;
        star2.manager = *manager.key;
        star3.manager = *manager.key;

        // Transfer from planet to star one and two
        ctx.accounts.star_one.add_lamports(star_one_amount)?;
        ctx.accounts.star_two.add_lamports(star_two_amount)?;
        ctx.accounts.star_three.add_lamports(star_three_amount)?;
        ctx.accounts.from_planet.sub_lamports(deposit.lamports)?;
        Ok(())
    }


    pub fn star_hop_three_end(ctx: Context<StarHopThreeEnd>) -> Result<()>{

        // Deposit account
        let deposit: &mut Account<Deposit> = &mut ctx.accounts.deposit;

        let to: &mut Account<Planet> = &mut ctx.accounts.to_planet;
        //let to_planet_name: String = to.name.to_owned();
        let star1: &mut Account<Star> = &mut ctx.accounts.star_one;
        let star2: &mut Account<Star> = &mut ctx.accounts.star_two;
        let star3: &mut Account<Star> = &mut ctx.accounts.star_three;
        let star_one_amount: u64 = star1.amount.clone();
        let star_two_amount: u64 = star2.amount.clone();
        let star_three_amount: u64 = star3.amount.clone();

        require!(star_one_amount + star_two_amount + star_three_amount == deposit.lamports, OridionError::StarHopCalculationError);

        // Update deposit location
        deposit.location = to.name.clone();
        hop_deposit(deposit);

         //Clear our star amount
        star1.amount = 0;
        star2.amount = 0;
        star3.amount = 0;
     
        //Increment planet visit
        to.visits += 1;
    
        //msg!("Star 1: {}", star_one_amount.to_string());
        //msg!("Star 2: {}", star_two_amount.to_string());
        //msg!("Star 3: {}", star_three_amount.to_string());

        // Transactions
        ctx.accounts.star_one.sub_lamports(star_one_amount)?;
        ctx.accounts.star_two.sub_lamports(star_two_amount)?;
        ctx.accounts.star_three.sub_lamports(star_three_amount)?;
        ctx.accounts.to_planet.add_lamports(deposit.lamports)?;
        //msg!("Hop to {} complete", to_planet_name);

        //Transfer out remaining lamports
        let star_one_remaining_lamports = ctx.accounts.star_one.get_lamports();
        let star_two_remaining_lamports = ctx.accounts.star_two.get_lamports();
        let star_three_remaining_lamports = ctx.accounts.star_three.get_lamports();
        ctx.accounts.manager.add_lamports(star_one_remaining_lamports)?;
        ctx.accounts.star_one.sub_lamports(star_one_remaining_lamports)?;
        
        ctx.accounts.manager.add_lamports(star_two_remaining_lamports)?;
        ctx.accounts.star_two.sub_lamports(star_two_remaining_lamports)?;

        ctx.accounts.manager.add_lamports(star_three_remaining_lamports)?;
        ctx.accounts.star_three.sub_lamports(star_three_remaining_lamports)?;

        Ok(())
    }
}