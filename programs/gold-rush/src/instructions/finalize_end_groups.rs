use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;
use anchor_lang::AccountDeserialize;

#[derive(Accounts)]
pub struct FinalizeEndGroups<'info> {
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

    pub system_program: Program<'info, System>,
}

impl<'info> FinalizeEndGroups<'info> {
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

        require!(
            self.round.captured_end_groups < self.round.total_groups,
            GoldRushError::GroupAssetAlreadyCapturedEndPrice
        );

        // Only allow finalize once
        require!(
            self.round.winner_group_ids.is_empty(),
            GoldRushError::SettlementFailed
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<FinalizeEndGroups>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let remaining_accounts = &ctx.remaining_accounts;

    // validate remaining accounts
    require!(
        !remaining_accounts.is_empty(),
        GoldRushError::InvalidRemainingAccountsLength
    );
    require!(
        remaining_accounts.len() <= MAX_REMAINING_ACCOUNTS,
        GoldRushError::InvalidRemainingAccountsLength
    );

    let round = &mut ctx.accounts.round;

    let mut max_avg: Option<i64> = None;
    let mut winner_ids: Vec<u64> = Vec::new();

    for acc_info in remaining_accounts.iter() {
        // Ownership must be our program (GroupAsset PDA)
        require_keys_eq!(
            *acc_info.owner,
            *ctx.program_id,
            GoldRushError::InvalidAssetAccount
        );

        // Borrow and deserialize GroupAsset
        let data = acc_info.try_borrow_data()?;
        let mut slice: &[u8] = &data;
        let group_asset: GroupAsset = GroupAsset::try_deserialize(&mut slice)
            .map_err(|_| GoldRushError::InvalidAssetAccountData)?;

        // Validate expected GroupAsset PDA
        let expected_pda = Pubkey::find_program_address(
            &[
                GROUP_ASSET_SEED.as_bytes(),
                round.key().as_ref(),
                &group_asset.id.to_le_bytes(),
            ],
            ctx.program_id,
        )
        .0;
        require_keys_eq!(
            *acc_info.key,
            expected_pda,
            GoldRushError::InvalidAssetAccount
        );

        // Must belong to this round
        require_keys_eq!(
            group_asset.round,
            round.key(),
            GoldRushError::InvalidAssetAccount
        );

        // Ensure group is fully finalized and has avg
        require!(
            group_asset.finalized_end_price_assets >= group_asset.total_assets,
            GoldRushError::SettlementFailed
        );
        let avg = group_asset
            .avg_growth_rate_bps
            .ok_or(GoldRushError::SettlementFailed)?;

        match max_avg {
            None => {
                max_avg = Some(avg);
                winner_ids.clear();
                winner_ids.push(group_asset.id);
            }
            Some(current_max) => {
                if avg > current_max {
                    max_avg = Some(avg);
                    winner_ids.clear();
                    winner_ids.push(group_asset.id);
                } else if avg == current_max {
                    winner_ids.push(group_asset.id);
                }
            }
        }
    }

    // winners not exceed limit
    require!(
        winner_ids.len() <= MAX_WINNER_GROUP_IDS,
        GoldRushError::SettlementFailed
    );

    // Set winners on the round
    round.winner_group_ids = winner_ids;

    Ok(())
}
