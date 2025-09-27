use crate::{constants::*, error::GoldRushError, state::*, utils::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CaptureStartPrice<'info> {
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

impl<'info> CaptureStartPrice<'info> {
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

pub fn handler(ctx: Context<CaptureStartPrice>) -> Result<()> {
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

    let round = &mut ctx.accounts.round;
    let group_asset = &mut ctx.accounts.group_asset;
    let current_timestamp = Clock::get()?.unix_timestamp;

    for pair in remaining_accounts.chunks(2) {
        let asset_ai = &pair[0];
        let pyth_ai = &pair[1];

        // ownership check
        require_keys_eq!(
            *asset_ai.owner,
            *ctx.program_id,
            GoldRushError::InvalidAssetAccount
        );

        let mut data = asset_ai.try_borrow_mut_data()?;
        let mut asset: Asset = Asset::try_deserialize(&mut &data[..])
            .map_err(|_| GoldRushError::InvalidAssetAccountData)?;

        // validate expected PDA
        let expected_pda = Pubkey::find_program_address(
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
            expected_pda,
            GoldRushError::InvalidAssetAccount
        );
        require_keys_eq!(
            asset.group,
            group_asset.key(),
            GoldRushError::InvalidAssetAccount
        );
        require_keys_eq!(asset.round, round.key(), GoldRushError::InvalidAssetAccount);

        // validate price feed
        require_keys_eq!(
            asset.price_feed_account,
            *pyth_ai.key,
            GoldRushError::InvalidPriceFeedAccount
        );

        // load price feed
        let normalized = load_pyth_price_normalized(
            pyth_ai,
            current_timestamp,
            ASSET_PRICE_STALENESS_THRESHOLD_SECONDS,
        )?;

        // set asset fields
        if asset.start_price.is_none() {
            require!(normalized > 0, GoldRushError::InvalidAssetPrice);
            asset.start_price = Some(normalized);

            // serialize back
            let serialized = asset
                .try_to_vec()
                .map_err(|_| GoldRushError::SerializeError)?;
            if serialized.len() > data[8..].len() {
                return Err(GoldRushError::AccountDataTooSmall.into());
            }
            data[8..8 + serialized.len()].copy_from_slice(&serialized);
        }

        // serialize
        let serialized = asset
            .try_to_vec()
            .map_err(|_| GoldRushError::SerializeError)?;
        if serialized.len() > data[8..].len() {
            return Err(GoldRushError::AccountDataTooSmall.into());
        }
        data[8..8 + serialized.len()].copy_from_slice(&serialized);
    }
    Ok(())
}
