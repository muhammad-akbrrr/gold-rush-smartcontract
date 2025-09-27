use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED.as_bytes()],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        seeds = [ROUND_SEED.as_bytes(), &round.id.to_le_bytes()],
        bump
    )]
    pub round: Account<'info, Round>,

    #[account(
        mut,
        seeds = [BET_SEED.as_bytes(), round.key().as_ref(), &bet.id.to_le_bytes()],
        bump
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

impl<'info> ClaimReward<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            matches!(
                self.config.status,
                ContractStatus::Active | ContractStatus::EmergencyPaused
            ),
            GoldRushError::ProgramPaused
        );

        require_keys_eq!(
            self.signer.key(),
            self.bet.bettor,
            GoldRushError::Unauthorized
        );

        require!(
            self.round.status == RoundStatus::Ended,
            GoldRushError::RoundNotEnded
        );

        require!(
            self.bet.status != BetStatus::Pending,
            GoldRushError::ClaimPendingBet
        );

        require!(
            self.bet.status != BetStatus::Lost,
            GoldRushError::ClaimLosingBet
        );

        require!(self.bet.claimed == false, GoldRushError::AlreadyClaimed);

        require_keys_eq!(
            self.round_vault.mint,
            self.mint.key(),
            GoldRushError::InvalidMint
        );
        require_keys_eq!(
            self.bettor_token_account.mint,
            self.mint.key(),
            GoldRushError::InvalidMint
        );

        if self.bet.status == BetStatus::Won {
            require!(
                self.round.winners_weight > 0,
                GoldRushError::RewardCalculationError
            );
        }

        Ok(())
    }
}

pub fn handler(ctx: Context<ClaimReward>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let bet = &mut ctx.accounts.bet;
    let round = &ctx.accounts.round;

    // calculate reward
    let reward_amount = match bet.status {
        BetStatus::Won => bet
            .weight
            .checked_mul(round.total_reward_pool)
            .and_then(|x| x.checked_div(round.winners_weight))
            .ok_or(GoldRushError::Underflow)?,
        BetStatus::Draw => bet.amount,
        BetStatus::Pending => return Err(GoldRushError::ClaimPendingBet.into()),
        BetStatus::Lost => return Err(GoldRushError::ClaimLosingBet.into()),
    };

    // transfer from vault to signer
    let transfer_accounts = Transfer {
        from: ctx.accounts.round_vault.to_account_info(),
        to: ctx.accounts.bettor_token_account.to_account_info(),
        authority: round.to_account_info(),
    };
    let round_id = round.id;
    let round_bump = round.bump;
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
    transfer(transfer_ctx, reward_amount)?;

    // set bet fields
    bet.claimed = true;

    Ok(())
}
