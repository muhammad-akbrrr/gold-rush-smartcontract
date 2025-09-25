use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InsertGroupAsset<'info> {
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
        bump,
    )]
    pub round: Account<'info, Round>,

    #[account(
        init,
        payer = signer,
        space = DISRIMINATOR_SIZE as usize + GroupAsset::INIT_SPACE,
        seeds = [GROUP_ASSET_SEED.as_bytes(), round.key().as_ref(), &(round.total_groups + 1).to_le_bytes()],
        bump
    )]
    pub group_asset: Account<'info, GroupAsset>,

    pub system_program: Program<'info, System>,
}

impl<'info> InsertGroupAsset<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            matches!(
                self.config.status,
                ContractStatus::Active | ContractStatus::EmergencyPaused,
            ),
            GoldRushError::ProgramPaused
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<InsertGroupAsset>, symbol: [u8; 8]) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let round = &mut ctx.accounts.round;
    let group_asset = &mut ctx.accounts.group_asset;

    // set group asset fields
    group_asset.id = round
        .total_groups
        .checked_add(1)
        .ok_or(GoldRushError::Overflow)?;
    group_asset.round = round.key();
    group_asset.symbol = symbol;
    group_asset.created_at = Clock::get()?.unix_timestamp;
    group_asset.bump = ctx.bumps.group_asset;

    // set round fields
    round.total_groups = round
        .total_groups
        .checked_add(1)
        .ok_or(GoldRushError::Overflow)?;

    Ok(())
}
