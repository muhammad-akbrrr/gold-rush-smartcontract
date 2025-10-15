use crate::{constants::*, error::GoldRushError, events::*, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct PauseProgram<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,
}

impl<'info> PauseProgram<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.config.status != ProgramStatus::Paused,
            GoldRushError::AlreadyPaused
        );

        require!(
            self.signer.key() == self.config.admin,
            GoldRushError::Unauthorized
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<PauseProgram>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let config = &mut ctx.accounts.config;

    // set fields
    config.status = ProgramStatus::Paused;

    // emit event
    emit!(ProgramPaused {
        admin: ctx.accounts.signer.key(),
        config: config.key(),
    });

    Ok(())
}
