use crate::{constants::*, error::GoldRushError, state::*, utils::*};
use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

#[derive(Accounts)]
pub struct CaptureEndPrice<'info> {
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

    #[account(
        mut,
        seeds = [GROUP_ASSET_SEED.as_bytes(), round.key().as_ref(), &group_asset.id.to_le_bytes()],
        bump
    )]
    pub group_asset: Account<'info, GroupAsset>,

    pub system_program: Program<'info, System>,
}

impl<'info> CaptureEndPrice<'info> {
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
            self.round.market_type == MarketType::GroupBattle,
            GoldRushError::InvalidRoundMarketType,
        );
        require!(
            Clock::get()?.unix_timestamp >= self.round.end_time,
            GoldRushError::RoundNotReadyForSettlement
        );

        require!(
            self.group_asset.captured_end_price_assets < self.group_asset.total_assets,
            GoldRushError::GroupAssetAlreadyCapturedEndPrice
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<CaptureEndPrice>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let remaining_accounts = &ctx.remaining_accounts;

    // validate remaining accounts
    require!(
        remaining_accounts.len() <= MAX_REMAINING_ACCOUNTS,
        GoldRushError::InvalidRemainingAccountsLength
    );
    require!(
        remaining_accounts.len() % 2 == 0,
        GoldRushError::InvalidRemainingAccountsLength
    );

    let config = &ctx.accounts.config;
    let round = &mut ctx.accounts.round;
    let group_asset = &mut ctx.accounts.group_asset;

    // if finalized price is not set, set it to current timestamp
    if group_asset.finalized_price_at.is_none() {
        group_asset.finalized_price_at = Some(Clock::get()?.unix_timestamp);
    }
    let finalized_ts = group_asset
        .finalized_price_at
        .ok_or(GoldRushError::InvalidAssetPrice)?;
    let finalized_price_at = Clock {
        unix_timestamp: finalized_ts,
        ..Clock::get()?
    };

    for pair in remaining_accounts.chunks(2) {
        let asset_ai = &pair[0];
        let pyth_ai = &pair[1];

        // ownership check
        require_keys_eq!(
            *asset_ai.owner,
            *ctx.program_id,
            GoldRushError::InvalidAssetAccount
        );

        // borrow and deserialize asset
        let mut asset_data = asset_ai.try_borrow_mut_data()?;
        let mut asset: Asset = Asset::try_deserialize(&mut &asset_data[..])
            .map_err(|_| GoldRushError::InvalidAssetAccountData)?;

        // validate asset PDA
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

        // borrow and deserialize price update
        let price_update_data = pyth_ai.try_borrow_data()?;
        let price_update: PriceUpdateV2 =
            PriceUpdateV2::try_deserialize(&mut &price_update_data[..])
                .map_err(|_| GoldRushError::InvalidPriceUpdateAccountData)?;

        // validate price feed
        require!(
            asset.feed_id == price_update.price_message.feed_id,
            GoldRushError::InvalidPriceFeedAccount
        );

        // load normalized price
        let price = price_update
            .get_price_no_older_than(
                &finalized_price_at,
                config.max_price_update_age_secs,
                &asset.feed_id,
            )
            .map_err(|_| GoldRushError::PythError)?;
        let normalized = normalize_price_to_u64(price.price, price.exponent)?;

        // set asset final price once
        if asset.final_price.is_none() {
            require!(normalized > 0, GoldRushError::InvalidAssetPrice);
            asset.final_price = Some(normalized);

            // serialize back
            let serialized = asset
                .try_to_vec()
                .map_err(|_| GoldRushError::SerializeError)?;
            if serialized.len() > asset_data[8..].len() {
                return Err(GoldRushError::AccountDataTooSmall.into());
            }
            asset_data[8..8 + serialized.len()].copy_from_slice(&serialized);
        }

        // serialize (idempotent)
        let serialized = asset
            .try_to_vec()
            .map_err(|_| GoldRushError::SerializeError)?;
        if serialized.len() > asset_data[8..].len() {
            return Err(GoldRushError::AccountDataTooSmall.into());
        }
        asset_data[8..8 + serialized.len()].copy_from_slice(&serialized);
    }
    Ok(())
}
