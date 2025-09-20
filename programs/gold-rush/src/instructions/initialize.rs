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
        keeper_authorities: Vec<Pubkey>,
        fee_gold_price_bps: u16,
        fee_stock_price_bps: u16,
        min_bet_amount: u64,
    ) -> Result<()> {
        require!(
            keeper_authorities.len() > 0,
            GoldRushError::InvalidKeeperAuthorities
        );

        require!(
            keeper_authorities.len() <= MAX_KEEPER_AUTHORITIES,
            GoldRushError::InvalidKeeperAuthorities
        );

        require!(fee_gold_price_bps <= MAX_FEE_BPS, GoldRushError::InvalidFee);

        require!(fee_stock_price_bps <= MAX_FEE_BPS, GoldRushError::InvalidFee);

        require!(min_bet_amount > 0, GoldRushError::InvalidMinBetAmount);

        Ok(())
    }
}

pub fn handler(
    ctx: Context<Initialize>,
    keeper_authorities: Vec<Pubkey>,
    token_mint: Pubkey,
    treasury: Pubkey,
    fee_gold_price_bps: u16,
    fee_stock_price_bps: u16,
    min_bet_amount: u64,
) -> Result<()> {
    // validate
    ctx.accounts.validate(
        keeper_authorities.clone(),
        fee_gold_price_bps,
        fee_stock_price_bps,
        min_bet_amount,
    )?;

    let config = &mut ctx.accounts.config;

    // set fields
    config.admin = ctx.accounts.signer.key();
    config.keeper_authorities = keeper_authorities;
    config.token_mint = token_mint;
    config.treasury = treasury;
    config.fee_gold_price_bps = fee_gold_price_bps;
    config.fee_stock_price_bps = fee_stock_price_bps;
    config.min_bet_amount = min_bet_amount;
    config.status = ContractStatus::Active;
    config.current_round_counter = 0;
    config.version = 0;
    config.bump = ctx.bumps.config;

    Ok(())
}
