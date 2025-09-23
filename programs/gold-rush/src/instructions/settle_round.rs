use crate::{constants::*, error::GoldRushError, state::*, utils::*};
use anchor_lang::prelude::*;
use anchor_lang::AccountDeserialize;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct SettleRound<'info> {
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

impl<'info> SettleRound<'info> {
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

        Ok(())
    }
}

pub fn handler(ctx: Context<SettleRound>, asset_price: u64) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let config = &ctx.accounts.config;
    let round = &mut ctx.accounts.round;

    // validate remaining accounts
    require!(
        ctx.remaining_accounts.len() <= MAX_BETS_SETTLE,
        GoldRushError::InvalidBettorsLength
    );

    if asset_price == 0 {
        if round.status == RoundStatus::Active {
            round.status = RoundStatus::PendingSettlement;
        }
    } else {
        let mut winners_weight = 0u64;

        // calculate price changed
        let locked_price = round.locked_price.unwrap();
        let price_change: i64 = (asset_price as i64)
            .checked_sub(locked_price as i64)
            .ok_or(GoldRushError::Overflow)?;

        for acc_info in ctx.remaining_accounts.iter() {
            // 1) ownership check
            require_keys_eq!(
                *acc_info.owner,
                *ctx.program_id,
                GoldRushError::InvalidBetAccount
            );

            // 2) borrow mut data
            let mut data = acc_info.try_borrow_mut_data()?;

            // 3) deserialize
            let mut bet: Bet = Bet::try_deserialize(&mut &data[..])
                .map_err(|_| GoldRushError::InvalidBetAccountData)?;

            // 4) validate expected PDA
            let expected_pda = Pubkey::find_program_address(
                &[
                    BET_SEED.as_bytes(),
                    round.key().as_ref(),
                    bet.bettor.as_ref(),
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

            // 5) decide win/loss/draw
            let is_winner = is_bet_winner(bet.direction.clone(), price_change);
            match is_winner {
                None => {
                    bet.status = BetStatus::Draw;
                }
                Some(true) => {
                    bet.status = BetStatus::Won;
                    winners_weight = winners_weight
                        .checked_add(bet.weight)
                        .ok_or(GoldRushError::Overflow)?;
                }
                Some(false) => {
                    bet.status = BetStatus::Lost;
                }
            }

            // 6) serialize back
            let serialized = bet
                .try_to_vec()
                .map_err(|_| GoldRushError::SerializeError)?;
            if serialized.len() > data[8..].len() {
                return Err(GoldRushError::AccountDataTooSmall.into());
            }
            data[8..8 + serialized.len()].copy_from_slice(&serialized);
        }

        // transfer fee
        let fee_bps = match round.market_type {
            MarketType::GoldPrice => config.fee_gold_price_bps,
            MarketType::StockPrice => config.fee_stock_price_bps,
        };
        let fee_amount = round
            .total_pool
            .checked_mul(fee_bps as u64)
            .and_then(|x| x.checked_div(HUNDRED_PERCENT_BPS as u64))
            .ok_or(GoldRushError::Overflow)?;
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

        // set round fields
        round.status = RoundStatus::Ended;
        round.final_price = Some(asset_price);
        round.winners_weight = winners_weight;
        round.total_fee_collected = fee_amount;
        round.settled_at = Some(Clock::get()?.unix_timestamp);
    }

    Ok(())
}
