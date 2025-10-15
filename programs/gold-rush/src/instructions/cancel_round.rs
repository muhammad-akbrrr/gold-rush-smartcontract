use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use anchor_spl::token::{close_account, transfer, CloseAccount, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct CancelRound<'info> {
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

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CancelRound<'info> {
    pub fn validate(&self) -> Result<()> {
        require!(
            matches!(
                self.config.status,
                ProgramStatus::Active | ProgramStatus::EmergencyPaused,
            ),
            GoldRushError::ProgramPaused
        );

        require!(
            self.signer.key() == self.config.admin,
            GoldRushError::Unauthorized
        );

        require!(
            self.treasury.key() == self.config.treasury,
            GoldRushError::InvalidTreasuryAuthority
        );

        require!(
            matches!(
                self.round.status,
                RoundStatus::Scheduled | RoundStatus::Active | RoundStatus::Cancelling
            ),
            GoldRushError::InvalidRoundStatus
        );

        Ok(())
    }
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CancelRound<'info>>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let round_id = ctx.accounts.round.id;
    let round_bump = ctx.accounts.round.bump;

    // validate remaining account
    require!(
        ctx.remaining_accounts.len() <= MAX_REMAINING_ACCOUNTS,
        GoldRushError::InvalidRemainingAccountsLength
    );
    require!(
        ctx.remaining_accounts.len() % 2 == 0,
        GoldRushError::InvalidRemainingAccountsLength
    );

    // if no bets at all
    if ctx.accounts.round.total_bets == 0 {
        // close round vault (authority = round PDA, seeds = ROUND)
        let close_vault_account = CloseAccount {
            account: ctx.accounts.round_vault.to_account_info(),
            destination: ctx.accounts.treasury.to_account_info(),
            authority: ctx.accounts.round.to_account_info(),
        };
        let round_seeds = &[
            ROUND_SEED.as_bytes(),
            &round_id.to_le_bytes(),
            &[round_bump],
        ];
        let round_signer = &[&round_seeds[..]];
        let close_vault_cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            close_vault_account,
            round_signer,
        );
        close_account(close_vault_cpi_ctx)?;

        // close round (AccountsClose pattern via system program not required)
        ctx.accounts
            .round
            .close(ctx.accounts.treasury.to_account_info())?;

        return Ok(());
    }

    // Set Cancelling to block actions during refunds
    {
        let round_mut = &mut ctx.accounts.round;
        round_mut.status = RoundStatus::Cancelling;
    }

    // Process bet refunds in pairs [Bet PDA, Bettor ATA]
    for i in (0..ctx.remaining_accounts.len()).step_by(2) {
        // 1) Read + validate bet (isolated)
        let bet_amount = {
            let bet_ai = &ctx.remaining_accounts[i];
            require_keys_eq!(
                *bet_ai.owner,
                *ctx.program_id,
                GoldRushError::InvalidBetAccount
            );
            let data = bet_ai.try_borrow_data()?;
            let mut data_slice: &[u8] = &data;
            let bet: Bet = Bet::try_deserialize(&mut data_slice)
                .map_err(|_| GoldRushError::InvalidBetAccountData)?;
            let expected_pda = Pubkey::find_program_address(
                &[
                    BET_SEED.as_bytes(),
                    ctx.accounts.round.key().as_ref(),
                    &bet.id.to_le_bytes(),
                ],
                ctx.program_id,
            )
            .0;
            require_keys_eq!(*bet_ai.key, expected_pda, GoldRushError::InvalidBetAccount);
            bet.amount
        };

        // 2) Refund (isolated CPI scope)
        if bet_amount > 0 {
            let token_program_info = ctx.accounts.token_program.to_account_info();
            let from_info = ctx.accounts.round_vault.to_account_info();
            let auth_info = ctx.accounts.round.to_account_info();
            let to_info = ctx.remaining_accounts[i + 1].to_account_info();

            let transfer_accounts = Transfer {
                from: from_info,
                to: to_info,
                authority: auth_info,
            };
            let round_seeds = &[
                ROUND_SEED.as_bytes(),
                &round_id.to_le_bytes(),
                &[round_bump],
            ];
            let signer = &[&round_seeds[..]];
            let transfer_ctx =
                CpiContext::new_with_signer(token_program_info, transfer_accounts, signer);
            transfer(transfer_ctx, bet_amount)?;
        }

        // 3) Update totals + close bet (isolated)
        {
            // update totals
            let round_mut = &mut ctx.accounts.round;
            round_mut.total_pool = round_mut
                .total_pool
                .checked_sub(bet_amount)
                .ok_or(GoldRushError::Underflow)?;
        }
        {
            // close bet account by manual lamports move + reassign
            let bet_ai = &ctx.remaining_accounts[i];
            let bet_lamports = bet_ai.lamports();
            **ctx.accounts.treasury.lamports.borrow_mut() = ctx
                .accounts
                .treasury
                .lamports()
                .checked_add(bet_lamports)
                .ok_or(GoldRushError::Overflow)?;
            **bet_ai.lamports.borrow_mut() = 0;
            bet_ai.assign(&system_program::ID);
        }
    }

    // accumulate cancelled bets
    {
        let round_mut = &mut ctx.accounts.round;
        round_mut.cancelled_bets = round_mut
            .cancelled_bets
            .checked_add((ctx.remaining_accounts.len() / 2) as u64)
            .ok_or(GoldRushError::Overflow)?;
    }

    // if all bets cancelled -> close vault + round
    if ctx.accounts.round.cancelled_bets >= ctx.accounts.round.total_bets {
        let close_vault_account = CloseAccount {
            account: ctx.accounts.round_vault.to_account_info(),
            destination: ctx.accounts.treasury.to_account_info(),
            authority: ctx.accounts.round.to_account_info(),
        };
        let round_seeds = &[
            ROUND_SEED.as_bytes(),
            &round_id.to_le_bytes(),
            &[round_bump],
        ];
        let round_signer = &[&round_seeds[..]];
        let close_vault_cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            close_vault_account,
            round_signer,
        );
        close_account(close_vault_cpi_ctx)?;

        ctx.accounts
            .round
            .close(ctx.accounts.treasury.to_account_info())?;
    }

    Ok(())
}
