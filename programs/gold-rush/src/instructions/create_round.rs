use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct CreateRound<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = signer,
        space = DISRIMINATOR_SIZE as usize + Round::INIT_SPACE,
        seeds = [ROUND_SEED.as_bytes(), &(config.current_round_counter + 1).to_le_bytes()],
        bump
    )]
    pub round: Account<'info, Round>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = round,
        seeds = [VAULT_SEED.as_bytes(), round.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> CreateRound<'info> {
    pub fn validate(&self, start_time: i64, end_time: i64) -> Result<()> {
        require!(
            self.config.status == ContractStatus::Active,
            GoldRushError::ProgramPaused
        );

        require!(
            self.signer.key() == self.config.admin,
            GoldRushError::Unauthorized
        );

        require!(start_time < end_time, GoldRushError::InvalidTimestamps);

        require!(
            start_time > Clock::get()?.unix_timestamp,
            GoldRushError::InvalidTimestamps
        );

        Ok(())
    }
}

pub fn handler(
    ctx: Context<CreateRound>,
    market_type: MarketType,
    start_time: i64,
    end_time: i64,
) -> Result<()> {
    // validate
    ctx.accounts.validate(start_time, end_time)?;

    let config = &mut ctx.accounts.config;
    let round = &mut ctx.accounts.round;

    // set fields
    round.id = config
        .current_round_counter
        .checked_add(1)
        .ok_or(GoldRushError::Overflow)?;
    round.start_time = start_time;
    round.end_time = end_time;
    // bet cutoff = end_time - config.bet_cutoff_window_secs (ensure it stays after start_time)
    let default_cutoff = end_time
        .checked_sub(config.bet_cutoff_window_secs)
        .ok_or(GoldRushError::Underflow)?;
    round.bet_cutoff_time = core::cmp::max(default_cutoff, start_time);
    round.vault = ctx.accounts.vault.key();
    round.vault_bump = ctx.bumps.vault;
    round.market_type = market_type;
    round.status = RoundStatus::Scheduled;
    round.created_at = Clock::get()?.unix_timestamp;
    round.bump = ctx.bumps.round;

    // set config fields
    config.current_round_counter = config
        .current_round_counter
        .checked_add(1)
        .ok_or(GoldRushError::Overflow)?;

    Ok(())
}
