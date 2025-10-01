use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;
use anchor_lang::AccountDeserialize;

#[derive(Accounts)]
pub struct FinalizeStartGroupAsset<'info> {
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

impl<'info> FinalizeStartGroupAsset<'info> {
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
            self.round.status == RoundStatus::Scheduled,
            GoldRushError::InvalidRoundStatus
        );
        require!(
            self.round.market_type == MarketType::GroupBattle,
            GoldRushError::InvalidRoundStatus
        );

        require_keys_eq!(
            self.group_asset.round,
            self.round.key(),
            GoldRushError::InvalidAssetAccount
        );

        require!(
            self.group_asset.finalized_start_price_assets < self.group_asset.total_assets,
            GoldRushError::GroupAssetAlreadyFinalizedStartPrice
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<FinalizeStartGroupAsset>) -> Result<()> {
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

    for asset_ai in remaining_accounts.iter() {
        // Ownership must be our program (Asset PDA)
        require_keys_eq!(
            *asset_ai.owner,
            *ctx.program_id,
            GoldRushError::InvalidAssetAccount
        );

        // Borrow and deserialize Asset
        let asset_data = asset_ai.try_borrow_data()?;
        let asset: Asset = Asset::try_deserialize(&mut &asset_data[..])
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
        require!(start_price > 0, GoldRushError::AssetStartPriceNotSet);
    }

    // Set group asset fields
    group_asset.finalized_start_price_assets = group_asset.total_assets;

    Ok(())
}
