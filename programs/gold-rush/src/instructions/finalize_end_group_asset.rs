use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;
use anchor_lang::AccountDeserialize;

#[derive(Accounts)]
pub struct FinalizeEndGroupAsset<'info> {
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

    #[account(
        mut,
        seeds = [GROUP_ASSET_SEED.as_bytes(), round.key().as_ref(), &group_asset.id.to_le_bytes()],
        bump
    )]
    pub group_asset: Account<'info, GroupAsset>,

    pub system_program: Program<'info, System>,
}

impl<'info> FinalizeEndGroupAsset<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            matches!(
                self.config.status,
                ContractStatus::Active | ContractStatus::EmergencyPaused,
            ),
            GoldRushError::ProgramPaused
        );

        require!(
            self.config.keeper_authorities.contains(&self.signer.key()),
            GoldRushError::UnauthorizedKeeper
        );

        require!(
            self.round.status == RoundStatus::Active,
            GoldRushError::InvalidRoundStatus
        );
        require!(
            matches!(self.round.market_type, MarketType::GroupBattle),
            GoldRushError::InvalidRoundMarketType
        );
        require!(
            Clock::get()?.unix_timestamp >= self.round.end_time,
            GoldRushError::RoundNotReadyForSettlement
        );

        require_keys_eq!(
            self.group_asset.round,
            self.round.key(),
            GoldRushError::InvalidAssetAccount
        );

        require!(
            self.group_asset.finalized_end_price_assets < self.group_asset.total_assets,
            GoldRushError::GroupAssetAlreadyFinalizedEndPrice
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<FinalizeEndGroupAsset>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let remaining_accounts = &ctx.remaining_accounts;

    require!(
        !remaining_accounts.is_empty(),
        GoldRushError::InvalidRemainingAccountsLength
    );
    require!(
        remaining_accounts.len() <= MAX_REMAINING_ACCOUNTS,
        GoldRushError::InvalidRemainingAccountsLength
    );

    let round = &ctx.accounts.round;
    let group_asset = &mut ctx.accounts.group_asset;

    let now = Clock::get()?.unix_timestamp;

    for asset_ai in remaining_accounts.iter() {
        // Ownership must be our program (Asset PDA)
        require_keys_eq!(
            *asset_ai.owner,
            *ctx.program_id,
            GoldRushError::InvalidAssetAccount
        );

        // Borrow and deserialize Asset
        let mut asset_data = asset_ai.try_borrow_mut_data()?;
        let mut asset: Asset = Asset::try_deserialize(&mut &asset_data[..])
            .map_err(|_| GoldRushError::InvalidAssetAccountData)?;

        // Validate asset PDA
        let expected_asset_pda = Pubkey::find_program_address(
            &[
                ASSET_SEED.as_bytes(),
                group_asset.key().as_ref(),
                &asset.id.to_le_bytes(),
            ],
            ctx.program_id,
        )
        .0;
        require_keys_eq!(
            *asset_ai.key,
            expected_asset_pda,
            GoldRushError::InvalidAssetAccount
        );
        require_keys_eq!(
            asset.group,
            group_asset.key(),
            GoldRushError::InvalidAssetAccount
        );
        require_keys_eq!(asset.round, round.key(), GoldRushError::InvalidAssetAccount);

        // Start price must be set
        let start_price = asset.start_price.ok_or(GoldRushError::InvalidAssetPrice)?;
        require!(start_price > 0, GoldRushError::InvalidAssetPrice);

        // Require final price already set by capture_end_price
        let final_price = asset.final_price.ok_or(GoldRushError::InvalidAssetPrice)?;
        require!(final_price > 0, GoldRushError::InvalidAssetPrice);

        // Compute signed growth bps with wide arithmetic
        let numerator: i128 = (final_price as i128)
            .checked_sub(start_price as i128)
            .ok_or(GoldRushError::Overflow)?;
        let growth_rate_bps_i128 = numerator
            .checked_mul(10_000)
            .ok_or(GoldRushError::Overflow)?
            .checked_div(start_price as i128)
            .ok_or(GoldRushError::Underflow)?;
        let growth_rate_bps: i64 =
            i64::try_from(growth_rate_bps_i128).map_err(|_| GoldRushError::Overflow)?;

        // Update asset
        asset.final_price = Some(final_price);
        asset.growth_rate_bps = Some(growth_rate_bps);
        asset.finalized_at = Some(now);

        // Serialize back
        let serialized = asset
            .try_to_vec()
            .map_err(|_| GoldRushError::SerializeError)?;
        if serialized.len() > asset_data[8..].len() {
            return Err(GoldRushError::AccountDataTooSmall.into());
        }
        asset_data[8..8 + serialized.len()].copy_from_slice(&serialized);
        group_asset.total_final_price = group_asset
            .total_final_price
            .checked_add(final_price)
            .ok_or(GoldRushError::Overflow)?;
        group_asset.total_growth_rate_bps = group_asset
            .total_growth_rate_bps
            .checked_add(growth_rate_bps)
            .ok_or(GoldRushError::Overflow)?;
        group_asset.finalized_end_price_assets = group_asset
            .finalized_end_price_assets
            .checked_add(1)
            .ok_or(GoldRushError::Overflow)?;
    }

    // Set group asset fields
    if group_asset.finalized_end_price_assets >= group_asset.total_assets {
        group_asset.avg_growth_rate_bps = Some(
            group_asset
                .total_growth_rate_bps
                .checked_div(group_asset.finalized_end_price_assets as i64)
                .ok_or(GoldRushError::Overflow)?,
        );
    }

    Ok(())
}
