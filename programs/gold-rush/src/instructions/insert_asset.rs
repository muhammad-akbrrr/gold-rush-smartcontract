use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

#[derive(Accounts)]
pub struct InsertAsset<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [ROUND_SEED.as_bytes(),&round.id.to_le_bytes()],
        bump,
    )]
    pub round: Account<'info, Round>,

    #[account(
        mut,
        seeds = [GROUP_ASSET_SEED.as_bytes(), round.key().as_ref(), &group_asset.id.to_le_bytes()],
        bump,
    )]
    pub group_asset: Account<'info, GroupAsset>,

    #[account(
        init,
        payer = signer,
        space = DISRIMINATOR_SIZE as usize + Asset::INIT_SPACE,
        seeds = [ASSET_SEED.as_bytes(), group_asset.key().as_ref(), &(group_asset.total_assets + 1).to_le_bytes()],
        bump,
    )]
    pub asset: Account<'info, Asset>,

    /// CHECK: This is the price feed account
    pub feed_price_account: Account<'info, PriceUpdateV2>,

    pub system_program: Program<'info, System>,
}

impl<'info> InsertAsset<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            matches!(
                self.config.status,
                ContractStatus::Active | ContractStatus::EmergencyPaused,
            ),
            GoldRushError::ProgramPaused
        );

        require!(
            self.signer.key() == self.config.admin,
            GoldRushError::Unauthorized
        );

        require!(
            self.group_asset.total_assets < MAX_GROUP_ASSETS as u64,
            GoldRushError::MaxAssetsReached
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<InsertAsset>, symbol: [u8; 8]) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let round = &ctx.accounts.round;
    let group_asset = &mut ctx.accounts.group_asset;
    let asset = &mut ctx.accounts.asset;

    // set asset fields
    asset.id = group_asset
        .total_assets
        .checked_add(1)
        .ok_or(GoldRushError::Overflow)?;
    asset.group = group_asset.key();
    asset.round = round.key();
    asset.feed_id = ctx.accounts.feed_price_account.price_message.feed_id;
    asset.symbol = symbol;
    asset.created_at = Clock::get()?.unix_timestamp;
    asset.bump = ctx.bumps.asset;

    // set group asset fields
    group_asset.total_assets = group_asset
        .total_assets
        .checked_add(1)
        .ok_or(GoldRushError::Overflow)?;

    Ok(())
}
