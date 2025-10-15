use crate::{constants::*, error::GoldRushError, events::*, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ProgramPause<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,
}

impl<'info> ProgramPause<'info> {
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

pub fn handler(ctx: Context<ProgramPause>) -> Result<()> {
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
