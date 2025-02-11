use anchor_lang::prelude::*;
use super::*;

///-------------------------------------------------------------//
/// BIG BANG UNIVERSE PDA
///-------------------------------------------------------------//
#[derive(Accounts)]
pub struct BigBang<'info> {
    #[account(
        init,
        payer = creator,
        space = Universe::LEN,
        seeds = [UNIVERSE_PDA_SEED],
        bump
    )]
    pub universe: Account<'info, Universe>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdateUniverseFee<'info> {
    #[account(mut)]
    pub universe: Account<'info, Universe>,
    #[account(mut, address = MANAGER_PUBKEY)]
    pub creator: Signer<'info>
}

#[account]
pub struct Universe {
    pub pda: Pubkey, //PDA
    pub p: Vec<String>, //Planets
    pub st: i64, //Universe started
    pub up: i64, //Last updated (used for comet random id)
    pub bp: u8, // Bump
    pub cfe: u64, // Comet Fee in lamports
    pub hpfe: u64, // Hop planet Fee in lamports
    pub hsfe2: u64, // Hop star Fee in lamports
    pub hsfe3: u64, // Hop star 3 Fee in lamports
    pub wfe: u64 // Withdraw Fee in lamports
}

impl Universe {
    const LEN: usize = DISCRIMINATOR_LENGTH
        + PUBLIC_KEY_LENGTH //Universe PDA
        + PLANETS_VEC_LENGTH // Planets vector (planets count can be counted here!)
        + TIMESTAMP_LENGTH // Universe started
        + TIMESTAMP_LENGTH // Last star generated
        + U8_LENGTH //Bump
        + LAMPORT_LENGTH // Comet fee
        + LAMPORT_LENGTH // Hop planet fee
        + LAMPORT_LENGTH // Hop star2 fee
        + LAMPORT_LENGTH // Hop star3 fee
        + LAMPORT_LENGTH; // Withdraw fee
}