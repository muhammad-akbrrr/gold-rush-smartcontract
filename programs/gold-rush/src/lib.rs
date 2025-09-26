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
        fee_gold_price_bps: u16,
        fee_stock_price_bps: u16,
        min_bet_amount: u64,
        min_time_factor_bps: u16,
        max_time_factor_bps: u16,
        default_direction_factor_bps: u16,
    ) -> Result<()> {
        initialize::handler(
            ctx,
            keeper_authorities,
            token_mint,
            treasury,
            fee_gold_price_bps,
            fee_stock_price_bps,
            min_bet_amount,
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
        new_fee_gold_price_bps: Option<u16>,
        new_fee_stock_price_bps: Option<u16>,
        new_min_bet_amount: Option<u64>,
    ) -> Result<()> {
        update_config::handler(
            ctx,
            new_admin,
            new_keeper_authorities,
            new_token_mint,
            new_treasury,
            new_fee_gold_price_bps,
            new_fee_stock_price_bps,
            new_min_bet_amount,
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

    pub fn capture_start_price(ctx: Context<CaptureStartPrice>) -> Result<()> {
        capture_start_price::handler(ctx)
    }

    pub fn start_round(ctx: Context<StartRound>) -> Result<()> {
        start_round::handler(ctx)
    }

    pub fn place_bet(ctx: Context<PlaceBet>, amount: u64, direction: BetDirection) -> Result<()> {
        place_bet::handler(ctx, amount, direction)
    }

    pub fn settle_round(ctx: Context<SettleRound>, asset_price: u64) -> Result<()> {
        settle_round::handler(ctx, asset_price)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        claim_reward::handler(ctx)
    }
}
