use crate::{constants::*, error::GoldRushError, events::*, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UnpauseProgram<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,
}

impl<'info> UnpauseProgram<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.config.status == ProgramStatus::Paused,
            GoldRushError::AlreadyActive
        );

        require!(
            self.signer.key() == self.config.admin,
            GoldRushError::Unauthorized
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<UnpauseProgram>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let config = &mut ctx.accounts.config;

    // set fields
    config.status = ProgramStatus::Active;

    // emit event
    emit!(ProgramUnpaused {
        admin: ctx.accounts.signer.key(),
        config: config.key(),
    });

    Ok(())
}
