use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;
use anchor_lang::AccountDeserialize;

#[derive(Accounts)]
pub struct FinalizeStartGroups<'info> {
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

impl<'info> FinalizeStartGroups<'info> {
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

        require!(
            self.round.captured_start_groups < self.round.total_groups,
            GoldRushError::GroupAssetAlreadyCapturedStartPrice
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<FinalizeStartGroups>) -> Result<()> {
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

    // TODO: Check semua group asset apakah sudah captured start price
    // kalau udah, update captured_start_groups di round

    let round = &mut ctx.accounts.round;

    for acc_info in remaining_accounts.iter() {
        // Ownership must be our program (GroupAsset PDA)
        require_keys_eq!(
            *acc_info.owner,
            *ctx.program_id,
            GoldRushError::InvalidAssetAccount
        );

        // Borrow and deserialize GroupAsset
        let data = acc_info.try_borrow_data()?;
        let group_asset: GroupAsset = GroupAsset::try_deserialize(&mut &data[..])
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

        // Set captured_start_groups on the round
        round.captured_start_groups = round
            .captured_start_groups
            .checked_add(1)
            .ok_or(GoldRushError::Overflow)?;
    }

    Ok(())
}
