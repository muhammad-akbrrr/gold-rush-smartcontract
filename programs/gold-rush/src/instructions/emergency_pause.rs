use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,
}

impl<'info> EmergencyPause<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.config.status != ContractStatus::EmergencyPaused,
            GoldRushError::AlreadyEmergencyPaused
        );

        require!(
            self.signer.key() == self.config.admin,
            GoldRushError::Unauthorized
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<EmergencyPause>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let config = &mut ctx.accounts.config;

    // set fields
    config.status = ContractStatus::EmergencyPaused;

    Ok(())
}
