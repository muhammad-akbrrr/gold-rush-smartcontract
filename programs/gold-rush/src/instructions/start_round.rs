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
        seeds = [ROUND_SEED.as_bytes(), &round.id.to_le_bytes()],
        bump
    )]
    pub round: Account<'info, Round>,

    pub system_program: Program<'info, System>,
}

impl<'info> StartRound<'info> {
    pub fn validate(&self, asset_price: u64) -> Result<()> {
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
            Clock::get()?.unix_timestamp > self.round.start_time,
            GoldRushError::RoundNotReady
        );

        require!(asset_price > 0, GoldRushError::InvalidAssetPrice);

        Ok(())
    }
}

pub fn handler(ctx: Context<StartRound>, asset_price: u64) -> Result<()> {
    // validate
    ctx.accounts.validate(asset_price)?;

    let round = &mut ctx.accounts.round;

    // set round fields
    round.status = RoundStatus::Active;
    round.locked_price = Some(asset_price);

    Ok(())
}
