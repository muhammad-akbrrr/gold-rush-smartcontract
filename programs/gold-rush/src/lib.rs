#![allow(unexpected_cfgs)]
#![allow(deprecated)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

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
    ) -> Result<()> {
        initialize::handler(
            ctx,
            keeper_authorities,
            token_mint,
            treasury,
            fee_gold_price_bps,
            fee_stock_price_bps,
            min_bet_amount,
        )
    }
}
