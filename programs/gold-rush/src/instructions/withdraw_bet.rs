use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct WithdrawBet<'info> {
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

    #[account(
        mut,
        close = signer,
    )]
    pub bet: Account<'info, Bet>,

    #[account(
        mut,
        seeds = [VAULT_SEED.as_bytes(), round.key().as_ref()],
        bump
    )]
    pub round_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
    )]
    pub bettor_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> WithdrawBet<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.config.status == ProgramStatus::Active,
            GoldRushError::ProgramPaused
        );

        require!(
            self.round.status == RoundStatus::Active,
            GoldRushError::RoundNotActive
        );

        require!(
            Clock::get()?.unix_timestamp < self.round.bet_cutoff_time,
            GoldRushError::RoundEnded
        );

        require_keys_eq!(
            self.bet.bettor,
            self.signer.key(),
            GoldRushError::Unauthorized
        );

        require_keys_eq!(
            self.bet.round,
            self.round.key(),
            GoldRushError::InvalidBetAccount
        );

        require!(
            self.bet.status == BetStatus::Pending,
            GoldRushError::InvalidBetStatus
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<WithdrawBet>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let round = &mut ctx.accounts.round;
    let bet = &ctx.accounts.bet;

    // Transfer tokens from round vault back to bettor
    let amount = bet.amount;
    if amount > 0 {
        let transfer_accounts = Transfer {
            from: ctx.accounts.round_vault.to_account_info(),
            to: ctx.accounts.bettor_token_account.to_account_info(),
            authority: round.to_account_info(),
        };
        let round_bump = round.bump;
        let round_id = round.id;
        let seeds = &[
            ROUND_SEED.as_bytes(),
            &round_id.to_le_bytes(),
            &[round_bump],
        ];
        let signer = &[&seeds[..]];
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
            signer,
        );
        transfer(transfer_ctx, amount)?;
    }

    // Update round aggregates
    round.total_pool = round
        .total_pool
        .checked_sub(bet.amount)
        .ok_or(GoldRushError::Underflow)?;
    round.total_bets = round
        .total_bets
        .checked_sub(1)
        .ok_or(GoldRushError::Underflow)?;

    // bet account will be closed automatically to bettor by close attribute
    Ok(())
}
