use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct StartRound<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [ROUND_SEED.as_bytes(), &round.id.to_le_bytes()],
        bump
    )]
    pub round: Account<'info, Round>,

    pub system_program: Program<'info, System>,
}

impl<'info> StartRound<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.config.status == ContractStatus::Active,
            GoldRushError::ProgramPaused
        );

        require!(
            self.config.keeper_authorities.contains(&self.signer.key()),
            GoldRushError::UnauthorizedKeeper
        );

        require!(
            self.round.status == RoundStatus::Scheduled,
            GoldRushError::InvalidRoundStatus
        );

        require!(
            Clock::get()?.unix_timestamp >= self.round.start_time,
            GoldRushError::RoundNotReady
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<StartRound>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let round = &mut ctx.accounts.round;

    // set round fields
    round.status = RoundStatus::Active;
    round.start_price = Some(asset_price);

    Ok(())
}
