use crate::{constants::*, error::GoldRushError, state::*, utils::*};
use anchor_lang::prelude::*;
use anchor_lang::AccountDeserialize;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct SettleSingleRound<'info> {
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
        seeds = [VAULT_SEED.as_bytes(), round.key().as_ref()],
        bump
    )]
    pub round_vault: Account<'info, TokenAccount>,

    /// CHECK: Treasury pubkey from config
    pub treasury: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = treasury,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> SettleSingleRound<'info> {
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
            self.treasury.key() == self.config.treasury,
            GoldRushError::InvalidTreasuryAuthority
        );

        // must be a SingleAsset round
        require!(
            matches!(self.round.market_type, MarketType::SingleAsset),
            GoldRushError::InvalidRoundStatus
        );

        require!(
            matches!(
                self.round.status,
                RoundStatus::Active | RoundStatus::PendingSettlement
            ),
            GoldRushError::InvalidRoundStatus
        );

        require!(
            Clock::get()?.unix_timestamp >= self.round.end_time,
            GoldRushError::RoundNotReadyForSettlement
        );

        // start price must exist from start_round
        require!(
            self.round.start_price.is_some(),
            GoldRushError::InvalidAssetPrice
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<SettleSingleRound>) -> Result<()> {
    // validate base constraints
    ctx.accounts.validate()?;

    let config = &ctx.accounts.config;
    let round = &mut ctx.accounts.round;

    // Read final price from oracle (first remaining account)
    require!(
        !ctx.remaining_accounts.is_empty(),
        GoldRushError::InvalidRemainingAccountsLength
    );
    let price_ai = &ctx.remaining_accounts[0];
    let now = Clock::get()?.unix_timestamp;
    let final_price =
        load_pyth_price_normalized(price_ai, now, ASSET_PRICE_STALENESS_THRESHOLD_SECONDS)?;
    require!(final_price > 0, GoldRushError::InvalidAssetPrice);

    // If no bets, end quickly
    if round.total_bets == 0 {
        round.status = RoundStatus::Ended;
        round.final_price = Some(final_price);
        round.settled_at = Some(now);
        return Ok(());
    }

    // If first batch, compute and lock fee and reward pool once
    if round.total_reward_pool == 0 && round.total_fee_collected == 0 {
        // For Single-Asset rounds, use fee_single_asset_bps
        let fee_bps = config.fee_single_asset_bps;
        let fee_amount = round
            .total_pool
            .checked_mul(fee_bps as u64)
            .and_then(|x| x.checked_div(HUNDRED_PERCENT_BPS as u64))
            .ok_or(GoldRushError::Overflow)?;
        round.total_fee_collected = fee_amount;
        round.total_reward_pool = round
            .total_pool
            .checked_sub(fee_amount)
            .ok_or(GoldRushError::Underflow)?;

        // Transfer fee to treasury
        if fee_amount > 0 {
            let transfer_accounts = Transfer {
                from: ctx.accounts.round_vault.to_account_info(),
                to: ctx.accounts.treasury_token_account.to_account_info(),
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
            transfer(transfer_ctx, fee_amount)?;
        }
    }

    // Determine price change
    let start_price = round.start_price.unwrap();
    let price_change: i64 = (final_price as i64)
        .checked_sub(start_price as i64)
        .ok_or(GoldRushError::Overflow)?;

    // Iterate over Bet PDAs in remaining accounts (batched)
    let mut batch_winners_weight = 0u64;
    for acc_info in ctx.remaining_accounts.iter().skip(1) {
        // Ownership must be our program (Bet PDA)
        require_keys_eq!(
            *acc_info.owner,
            *ctx.program_id,
            GoldRushError::InvalidBetAccount
        );

        // Borrow and deserialize Bet
        let mut data = acc_info.try_borrow_mut_data()?;
        let mut bet: Bet = Bet::try_deserialize(&mut &data[..])
            .map_err(|_| GoldRushError::InvalidBetAccountData)?;

        // Validate expected Bet PDA
        let expected_pda = Pubkey::find_program_address(
            &[
                BET_SEED.as_bytes(),
                round.key().as_ref(),
                &bet.id.to_le_bytes(),
            ],
            ctx.program_id,
        )
        .0;
        require_keys_eq!(
            *acc_info.key,
            expected_pda,
            GoldRushError::InvalidBetAccount
        );

        // Decide result
        let is_winner = is_bet_winner(bet.direction.clone(), price_change);
        match is_winner {
            None => {
                bet.status = BetStatus::Draw;
            }
            Some(true) => {
                bet.status = BetStatus::Won;
                batch_winners_weight = batch_winners_weight
                    .checked_add(bet.weight)
                    .ok_or(GoldRushError::Overflow)?;
            }
            Some(false) => {
                bet.status = BetStatus::Lost;
            }
        }

        // Serialize back
        let serialized = bet
            .try_to_vec()
            .map_err(|_| GoldRushError::SerializeError)?;
        if serialized.len() > data[8..].len() {
            return Err(GoldRushError::AccountDataTooSmall.into());
        }
        data[8..8 + serialized.len()].copy_from_slice(&serialized);
    }

    // Accumulate progress
    round.winners_weight = round
        .winners_weight
        .checked_add(batch_winners_weight)
        .ok_or(GoldRushError::Overflow)?;
    round.settled_bets = round
        .settled_bets
        .checked_add((ctx.remaining_accounts.len().saturating_sub(1)) as u64)
        .ok_or(GoldRushError::Overflow)?;

    // Finalize when all bets processed
    if round.settled_bets >= round.total_bets {
        round.status = RoundStatus::Ended;
        round.final_price = Some(final_price);
        round.settled_at = Some(now);
    } else if round.status == RoundStatus::Active {
        // mark as pending to indicate partial settlement in progress
        round.status = RoundStatus::PendingSettlement;
    }

    Ok(())
}
