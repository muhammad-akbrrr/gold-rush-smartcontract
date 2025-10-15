use crate::{constants::*, error::GoldRushError, events::*, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct EmergencyUnpause<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,
}

impl<'info> EmergencyUnpause<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.config.status == ProgramStatus::EmergencyPaused,
            GoldRushError::NotEmergencyPaused
        );

        require!(
            self.signer.key() == self.config.admin,
            GoldRushError::Unauthorized
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<EmergencyUnpause>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let config = &mut ctx.accounts.config;

    // set fields
    config.status = ProgramStatus::Active;

    // emit event
    emit!(EmergencyUnpaused {
        admin: ctx.accounts.signer.key(),
        config: config.key(),
    });

    Ok(())
}
