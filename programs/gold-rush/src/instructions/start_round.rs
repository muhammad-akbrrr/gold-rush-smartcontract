use crate::{constants::*, error::GoldRushError, state::*, utils::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct StartRound<'info> {
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

impl<'info> StartRound<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.config.status == ContractStatus::Active,
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
            Clock::get()?.unix_timestamp >= self.round.start_time,
            GoldRushError::RoundNotReady
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<StartRound>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let round = &mut ctx.accounts.round;

    // Single-Asset: validate by market type, take the price from the oracle
    if matches!(round.market_type, MarketType::SingleAsset) {
        // Expect at least 1 remaining account as the price feed account (e.g. Pyth)
        require!(
            !ctx.remaining_accounts.is_empty(),
            GoldRushError::InvalidRemainingAccountsLength
        );

        let price_ai = &ctx.remaining_accounts[0];
        let now = Clock::get()?.unix_timestamp;
        let price =
            load_pyth_price_normalized(price_ai, now, ASSET_PRICE_STALENESS_THRESHOLD_SECONDS)?;

        require!(price > 0, GoldRushError::InvalidAssetPrice);

        // set start price for single-asset
        round.start_price = Some(price);
    }

    // activate round
    round.status = RoundStatus::Active;

    Ok(())
}
