use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = DISRIMINATOR_SIZE as usize + Config::INIT_SPACE,
        seeds = [CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn validate(
        &self,
        keeper_authorities: &Vec<Pubkey>,
        fee_single_asset_bps: u16,
        fee_group_battle_bps: u16,
        min_bet_amount: u64,
        bet_cutoff_window_secs: i64,
        min_time_factor_bps: u16,
        max_time_factor_bps: u16,
        default_direction_factor_bps: u16,
    ) -> Result<()> {
        require!(
            keeper_authorities.len() > 0,
            GoldRushError::InvalidKeeperAuthorities
        );

        require!(
            keeper_authorities.len() <= MAX_KEEPER_AUTHORITIES,
            GoldRushError::InvalidKeeperAuthorities
        );

        require!(
            fee_single_asset_bps <= HUNDRED_PERCENT_BPS,
            GoldRushError::InvalidFee
        );

        require!(
            fee_group_battle_bps <= HUNDRED_PERCENT_BPS,
            GoldRushError::InvalidFee
        );

        require!(min_bet_amount > 0, GoldRushError::InvalidMinBetAmount);

        require!(
            bet_cutoff_window_secs >= 0,
            GoldRushError::InvalidNewBetCutoffWindowSecs
        );

        require!(
            (0..=HUNDRED_PERCENT_BPS).contains(&min_time_factor_bps),
            GoldRushError::InvalidTimeFactorConfig
        );

        require!(
            (0..=HUNDRED_PERCENT_BPS).contains(&max_time_factor_bps),
            GoldRushError::InvalidTimeFactorConfig
        );

        require!(
            (0..=HUNDRED_PERCENT_BPS).contains(&default_direction_factor_bps),
            GoldRushError::InvalidDirectionFactorConfig
        );

        require!(
            min_time_factor_bps <= max_time_factor_bps,
            GoldRushError::InvalidTimeFactorRange
        );

        Ok(())
    }
}

pub fn handler(
    ctx: Context<Initialize>,
    keeper_authorities: Vec<Pubkey>,
    token_mint: Pubkey,
    treasury: Pubkey,
    fee_single_asset_bps: u16,
    fee_group_battle_bps: u16,
    min_bet_amount: u64,
    bet_cutoff_window_secs: i64,
    min_time_factor_bps: u16,
    max_time_factor_bps: u16,
    default_direction_factor_bps: u16,
) -> Result<()> {
    // validate
    ctx.accounts.validate(
        &keeper_authorities,
        fee_single_asset_bps,
        fee_group_battle_bps,
        min_bet_amount,
        bet_cutoff_window_secs,
        min_time_factor_bps,
        max_time_factor_bps,
        default_direction_factor_bps,
    )?;

    let config = &mut ctx.accounts.config;

    // set fields
    config.admin = ctx.accounts.signer.key();
    config.keeper_authorities = keeper_authorities;
    config.token_mint = token_mint;
    config.treasury = treasury;
    config.fee_single_asset_bps = fee_single_asset_bps;
    config.fee_group_battle_bps = fee_group_battle_bps;
    config.min_bet_amount = min_bet_amount;
    config.bet_cutoff_window_secs = bet_cutoff_window_secs;
    config.min_time_factor_bps = min_time_factor_bps;
    config.max_time_factor_bps = max_time_factor_bps;
    config.default_direction_factor_bps = default_direction_factor_bps;
    config.status = ContractStatus::Active;
    config.bump = ctx.bumps.config;

    Ok(())
}
