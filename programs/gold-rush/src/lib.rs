#![allow(unexpected_cfgs)]
#![allow(deprecated)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("FM7SQyRJExhzjFYvZ6XZTLkSSjNcMdDkCq89PWF9FtMB");

#[program]
pub mod gold_rush {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        keeper_authorities: Vec<Pubkey>,
        token_mint: Pubkey,
        treasury: Pubkey,
        single_asset_feed_id: [u8; 32],
        max_price_update_age_secs: u64,
        fee_single_asset_bps: u16,
        fee_group_battle_bps: u16,
        min_bet_amount: u64,
        bet_cutoff_window_secs: i64,
        min_time_factor_bps: u16,
        max_time_factor_bps: u16,
        default_direction_factor_bps: u16,
    ) -> Result<()> {
        initialize::handler(
            ctx,
            keeper_authorities,
            token_mint,
            treasury,
            single_asset_feed_id,
            max_price_update_age_secs,
            fee_single_asset_bps,
            fee_group_battle_bps,
            min_bet_amount,
            bet_cutoff_window_secs,
            min_time_factor_bps,
            max_time_factor_bps,
            default_direction_factor_bps,
        )
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        new_admin: Option<Pubkey>,
        new_keeper_authorities: Option<Vec<Pubkey>>,
        new_token_mint: Option<Pubkey>,
        new_treasury: Option<Pubkey>,
        new_single_asset_feed_id: Option<[u8; 32]>,
        new_max_price_update_age_secs: Option<u64>,
        new_fee_single_asset_bps: Option<u16>,
        new_fee_group_battle_bps: Option<u16>,
        new_min_bet_amount: Option<u64>,
        new_bet_cutoff_window_secs: Option<i64>,
    ) -> Result<()> {
        update_config::handler(
            ctx,
            new_admin,
            new_keeper_authorities,
            new_token_mint,
            new_treasury,
            new_single_asset_feed_id,
            new_max_price_update_age_secs,
            new_fee_single_asset_bps,
            new_fee_group_battle_bps,
            new_min_bet_amount,
            new_bet_cutoff_window_secs,
        )
    }

    pub fn pause_program(ctx: Context<PauseProgram>) -> Result<()> {
        pause_program::handler(ctx)
    }

    pub fn unpause_program(ctx: Context<UnpauseProgram>) -> Result<()> {
        unpause_program::handler(ctx)
    }

    pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
        emergency_pause::handler(ctx)
    }

    pub fn emergency_unpause(ctx: Context<EmergencyUnpause>) -> Result<()> {
        emergency_unpause::handler(ctx)
    }

    pub fn create_round(
        ctx: Context<CreateRound>,
        market_type: MarketType,
        start_time: i64,
        end_time: i64,
    ) -> Result<()> {
        create_round::handler(ctx, market_type, start_time, end_time)
    }

    pub fn insert_group_asset(ctx: Context<InsertGroupAsset>, symbol: [u8; 8]) -> Result<()> {
        insert_group_asset::handler(ctx, symbol)
    }

    pub fn insert_asset(ctx: Context<InsertAsset>, symbol: [u8; 8]) -> Result<()> {
        insert_asset::handler(ctx, symbol)
    }

    pub fn cancel_round<'info>(ctx: Context<'_, '_, '_, 'info, CancelRound<'info>>) -> Result<()> {
        cancel_round::handler(ctx)
    }

    pub fn capture_start_price(ctx: Context<CaptureStartPrice>) -> Result<()> {
        capture_start_price::handler(ctx)
    }

    pub fn start_round<'info>(ctx: Context<'_, '_, 'info, 'info, StartRound<'info>>) -> Result<()> {
        start_round::handler(ctx)
    }

    pub fn place_bet(ctx: Context<PlaceBet>, amount: u64, direction: BetDirection) -> Result<()> {
        place_bet::handler(ctx, amount, direction)
    }

    pub fn withdraw_bet(ctx: Context<WithdrawBet>) -> Result<()> {
        withdraw_bet::handler(ctx)
    }

    pub fn capture_end_price(ctx: Context<CaptureEndPrice>) -> Result<()> {
        capture_end_price::handler(ctx)
    }

    pub fn finalize_group_asset(ctx: Context<FinalizeGroupAsset>) -> Result<()> {
        finalize_group_asset::handler(ctx)
    }

    pub fn finalize_groups(ctx: Context<FinalizeGroups>) -> Result<()> {
        finalize_groups::handler(ctx)
    }

    pub fn settle_single_round<'info>(
        ctx: Context<'_, '_, 'info, 'info, SettleSingleRound<'info>>,
    ) -> Result<()> {
        settle_single_round::handler(ctx)
    }

    pub fn settle_group_round(ctx: Context<SettleGroupRound>) -> Result<()> {
        settle_group_round::handler(ctx)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        claim_reward::handler(ctx)
    }
}
