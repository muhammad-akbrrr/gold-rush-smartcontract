use crate::{constants::*, error::GoldRushError, state::*, utils::*};
use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

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
            GoldRushError::RoundNotReadyForStart
        );

        if matches!(self.round.market_type, MarketType::GroupBattle) {
            require!(
                self.round.captured_start_groups >= self.round.total_groups,
                GoldRushError::RoundNotCapturedStartPrice
            );
        }

        Ok(())
    }
}

pub fn handler<'info>(ctx: Context<'_, '_, 'info, 'info, StartRound<'info>>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let config = &ctx.accounts.config;
    let round = &mut ctx.accounts.round;

    if matches!(round.market_type, MarketType::SingleAsset) {
        require!(
            !ctx.remaining_accounts.is_empty(),
            GoldRushError::InvalidRemainingAccountsLength
        );

        let price_update: Account<PriceUpdateV2> = Account::try_from(&ctx.remaining_accounts[0])
            .map_err(|_| GoldRushError::InvalidPriceUpdateAccountData)?;
        let price = price_update
            .get_price_no_older_than(
                &Clock::get()?,
                config.max_price_update_age_secs,
                &config.single_asset_feed_id,
            )
            .map_err(|_| GoldRushError::PythError)?;

        let normalized = normalize_price_to_u64(price.price, price.exponent)?;
        require!(normalized > 0, GoldRushError::InvalidAssetPrice);

        // set start price for single-asset
        round.start_price = Some(normalized);
    }

    // activate round
    round.status = RoundStatus::Active;

    Ok(())
}
