use crate::{constants::*, error::GoldRushError, state::*, utils::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct PlaceBet<'info> {
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

    // Optional: only required for GroupBattle rounds
    pub group_asset: Option<Account<'info, GroupAsset>>,

    #[account(
        init,
        payer = signer,
        space = DISRIMINATOR_SIZE as usize + Bet::INIT_SPACE,
        seeds = [BET_SEED.as_bytes(), round.key().as_ref(), &(round.total_bets + 1).to_le_bytes()],
        bump
    )]
    pub bet: Account<'info, Bet>,

    #[account(
        mut,
        seeds = [VAULT_SEED.as_bytes(), round.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer
    )]
    pub token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> PlaceBet<'info> {
    pub fn validate(&self, amount: u64) -> Result<()> {
        require!(
            self.config.status == ContractStatus::Active,
            GoldRushError::ProgramPaused
        );

        require!(
            self.round.status == RoundStatus::Active,
            GoldRushError::RoundNotActive
        );

        // Enforce bet cutoff rather than the exact end_time to prevent last-second betting
        require!(
            Clock::get()?.unix_timestamp < self.round.bet_cutoff_time,
            GoldRushError::RoundEnded
        );

        // For GroupBattle, group_asset must be provided and belong to this round
        if matches!(self.round.market_type, MarketType::GroupBattle) {
            let ga = self
                .group_asset
                .as_ref()
                .ok_or(GoldRushError::InvalidAssetAccount)?;
            require_keys_eq!(
                ga.round,
                self.round.key(),
                GoldRushError::InvalidAssetAccount
            );
        }

        require!(
            amount >= self.config.min_bet_amount,
            GoldRushError::BetBelowMinimum
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<PlaceBet>, amount: u64, direction: BetDirection) -> Result<()> {
    // validate
    ctx.accounts.validate(amount)?;

    // transfer from signer to vault
    let transfer_accounts = Transfer {
        from: ctx.accounts.token_account.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts,
    );
    transfer(transfer_ctx, amount)?;

    let config = &ctx.accounts.config;
    let round = &mut ctx.accounts.round;
    let bet = &mut ctx.accounts.bet;

    // Assign group only for GroupBattle
    if matches!(round.market_type, MarketType::GroupBattle) {
        let ga = ctx
            .accounts
            .group_asset
            .as_ref()
            .ok_or(GoldRushError::InvalidAssetAccount)?;
        bet.group = Some(ga.key());
    }

    // calculate bet weight
    let round_duration = round
        .end_time
        .checked_sub(round.start_time)
        .ok_or(GoldRushError::Underflow)?;
    let now = Clock::get()?.unix_timestamp;
    let time_elapsed = now
        .checked_sub(round.start_time)
        .ok_or(GoldRushError::Underflow)?;
    let direction_factor = calculate_direction_factor(
        &round.market_type,
        &direction,
        config.default_direction_factor_bps as u64,
    )?;
    let time_factor = calculate_time_factor(
        &round.market_type,
        time_elapsed,
        config.min_time_factor_bps as u64,
        config.max_time_factor_bps as u64,
        round_duration,
    )?;
    let numerator = (amount as u128)
        .checked_mul(direction_factor as u128)
        .ok_or(GoldRushError::Overflow)?
        .checked_mul(time_factor as u128)
        .ok_or(GoldRushError::Overflow)?;
    let denominator = (HUNDRED_PERCENT_BPS as u128)
        .checked_mul(HUNDRED_PERCENT_BPS as u128)
        .ok_or(GoldRushError::Overflow)?;
    let weight = numerator
        .checked_div(denominator)
        .ok_or(GoldRushError::Underflow)? as u64;

    // set bet fields
    bet.id = round.total_bets + 1;
    bet.round = round.key();
    bet.bettor = ctx.accounts.signer.key();
    bet.amount = amount;
    bet.direction = direction;
    bet.weight = weight;
    bet.status = BetStatus::Pending;
    bet.created_at = Clock::get()?.unix_timestamp;
    bet.bump = ctx.bumps.bet;

    // set round fields
    round.total_pool = round
        .total_pool
        .checked_add(amount)
        .ok_or(GoldRushError::Overflow)?;
    round.total_bets = round
        .total_bets
        .checked_add(1)
        .ok_or(GoldRushError::Overflow)?;

    Ok(())
}
