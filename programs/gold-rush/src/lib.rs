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

    pub fn create_round(
        ctx: Context<CreateRound>,
        market_type: MarketType,
        start_time: i64,
        end_time: i64,
    ) -> Result<()> {
        create_round::handler(ctx, market_type, start_time, end_time)
    }

    pub fn start_round(ctx: Context<StartRound>, asset_price: u64) -> Result<()> {
        start_round::handler(ctx, asset_price)
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
