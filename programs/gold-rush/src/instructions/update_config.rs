use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,
}

impl<'info> UpdateConfig<'info> {
    pub fn validate(
        &self,
        new_admin: Option<Pubkey>,
        new_keeper_authorities: &Option<Vec<Pubkey>>,
        new_token_mint: Option<Pubkey>,
        new_treasury: Option<Pubkey>,
        new_fee_single_asset_bps: Option<u16>,
        new_fee_group_battle_bps: Option<u16>,
        new_min_bet_amount: Option<u64>,
    ) -> Result<()> {
        require!(
            self.signer.key() == self.config.admin,
            GoldRushError::Unauthorized
        );

        if let Some(new_admin) = new_admin {
            require!(
                new_admin != self.config.admin,
                GoldRushError::InvalidNewAdmin
            );
        }

        if let Some(new_keeper_authorities) = new_keeper_authorities {
            require!(
                new_keeper_authorities.len() > 0,
                GoldRushError::NoNewKeeperAuthorities
            );

            require!(
                new_keeper_authorities.len() <= MAX_KEEPER_AUTHORITIES as usize,
                GoldRushError::MaxKeeperAuthoritiesReached
            );
        }

        if let Some(new_token_mint) = new_token_mint {
            require!(
                new_token_mint != self.config.token_mint,
                GoldRushError::InvalidNewTokenMint
            );
        }

        if let Some(new_treasury) = new_treasury {
            require!(
                new_treasury != self.config.treasury,
                GoldRushError::InvalidNewTreasury
            );
        }

        if let Some(new_fee_single_asset_bps) = new_fee_single_asset_bps {
            require!(
                new_fee_single_asset_bps < HUNDRED_PERCENT_BPS,
                GoldRushError::InvalidNewFeeGoldPriceBps
            );
        }

        if let Some(new_fee_group_battle_bps) = new_fee_group_battle_bps {
            require!(
                new_fee_group_battle_bps < HUNDRED_PERCENT_BPS,
                GoldRushError::InvalidNewFeeStockPriceBps
            );
        }

        if let Some(new_min_bet_amount) = new_min_bet_amount {
            require!(
                new_min_bet_amount > 0,
                GoldRushError::InvalidNewMinBetAmount
            );
        }

        Ok(())
    }
}

pub fn handler(
    ctx: Context<UpdateConfig>,
    new_admin: Option<Pubkey>,
    new_keeper_authorities: Option<Vec<Pubkey>>,
    new_token_mint: Option<Pubkey>,
    new_treasury: Option<Pubkey>,
    new_fee_single_asset_bps: Option<u16>,
    new_fee_group_battle_bps: Option<u16>,
    new_min_bet_amount: Option<u64>,
) -> Result<()> {
    // validate
    ctx.accounts.validate(
        new_admin,
        &new_keeper_authorities,
        new_token_mint,
        new_treasury,
        new_fee_single_asset_bps,
        new_fee_group_battle_bps,
        new_min_bet_amount,
    )?;

    let config = &mut ctx.accounts.config;

    // set fields
    if let Some(new_admin) = new_admin {
        config.admin = new_admin;
    }
    if let Some(new_keeper_authorities) = new_keeper_authorities {
        config.keeper_authorities = new_keeper_authorities;
    }
    if let Some(new_token_mint) = new_token_mint {
        config.token_mint = new_token_mint;
    }
    if let Some(new_treasury) = new_treasury {
        config.treasury = new_treasury;
    }
    if let Some(new_fee_single_asset_bps) = new_fee_single_asset_bps {
        config.fee_single_asset_bps = new_fee_single_asset_bps;
    }
    if let Some(new_fee_group_battle_bps) = new_fee_group_battle_bps {
        config.fee_group_battle_bps = new_fee_group_battle_bps;
    }
    if let Some(new_min_bet_amount) = new_min_bet_amount {
        config.min_bet_amount = new_min_bet_amount;
    }

    // update config version
    config.version = config
        .version
        .checked_add(1)
        .ok_or(GoldRushError::Overflow)?;

    Ok(())
}
