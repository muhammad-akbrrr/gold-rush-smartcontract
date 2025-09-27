use crate::{constants::*, error::GoldRushError, state::*};
use anchor_lang::prelude::*;
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
                ContractStatus::Active | ContractStatus::EmergencyPaused,
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

pub fn handler(ctx: Context<CancelRound>) -> Result<()> {
    // validate
    ctx.accounts.validate()?;

    let round = &mut ctx.accounts.round;
    let round_vault = &mut ctx.accounts.round_vault;
    let remaining_accounts = &ctx.remaining_accounts;

    let round_id = round.id;

    // validate remaining account
    require!(
        remaining_accounts.len() <= MAX_REMAINING_ACCOUNTS,
        GoldRushError::InvalidRemainingAccountsLength
    );
    require!(
        remaining_accounts.len() % 2 == 0,
        GoldRushError::InvalidRemainingAccountsLength
    );

    // if not bets at all
    if round.total_bets == 0 {
        // close round vault
        let close_vault_account = CloseAccount {
            account: round_vault.to_account_info(),
            destination: ctx.accounts.treasury.to_account_info(),
            authority: round.to_account_info(),
        };
        let vault_seeds = &[
            VAULT_SEED.as_bytes(),
            &round_id.to_le_bytes(),
            &[round.vault_bump],
        ];
        let vault_signer = &[&vault_seeds[..]];
        let close_vault_cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            close_vault_account,
            vault_signer,
        );
        close_account(close_vault_cpi_ctx)?;

        // close round
        let close_round_account = CloseAccount {
            account: round.to_account_info(),
            destination: ctx.accounts.treasury.to_account_info(),
            authority: round.to_account_info(),
        };
        let round_seeds = &[
            ROUND_SEED.as_bytes(),
            &round_id.to_le_bytes(),
            &[round.bump],
        ];
        let round_signer = &[&round_seeds[..]];
        let close_round_cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            close_round_account,
            round_signer,
        );
        close_account(close_round_cpi_ctx)?;

        return Ok(());
    }

    // TODO: processing bet accounts
    // for pair in ctx.remaining_accounts.chunks(2) {
    //     let bet_ai = &pair[0];
    //     let token_ai = &pair[1];

    //     // ownership check
    //     require_keys_eq!(
    //         *bet_ai.owner,
    //         *ctx.program_id,
    //         GoldRushError::InvalidBetAccount
    //     );

    //     let data = bet_ai.try_borrow_data()?;
    //     let mut data_slice: &[u8] = &data;
    //     let bet: Bet = Bet::try_deserialize(&mut data_slice)
    //         .map_err(|_| GoldRushError::InvalidBetAccountData)?;

    //     // validate bet account
    //     let expected_pda = Pubkey::find_program_address(
    //         &[
    //             BET_SEED.as_bytes(),
    //             ctx.accounts.round.key().as_ref(),
    //             &bet.id.to_le_bytes(),
    //         ],
    //         ctx.program_id,
    //     )
    //     .0;
    //     require_keys_eq!(*bet_ai.key, expected_pda, GoldRushError::InvalidBetAccount);

    //     // transfer amount from vault to bettor
    //     let transfer_accounts = Transfer {
    //         from: ctx.accounts.round_vault.to_account_info(),
    //         to: token_ai.to_account_info(),
    //         authority: ctx.accounts.round.to_account_info(),
    //     };
    //     let transfer_cpi_ctx = CpiContext::new(
    //         ctx.accounts.token_program.to_account_info(),
    //         transfer_accounts,
    //     );
    //     transfer(transfer_cpi_ctx, bet.amount)?;

    //     // close bet account
    //     // let close_bet_account = CloseAccount {
    //     //     account: bet_ai.to_account_info(),
    //     //     destination: token_ai.to_account_info(),
    //     //     authority: signer.to_account_info(),
    //     // };
    //     // let round_key = round.key();
    //     // let bet_seeds = &[
    //     //     BET_SEED.as_bytes(),
    //     //     round_key.as_ref(),
    //     //     &bet.id.to_le_bytes(),
    //     // ];
    //     // let bet_signer = &[&bet_seeds[..]];
    //     // let close_bet_cpi_ctx = CpiContext::new_with_signer(
    //     //     ctx.accounts.system_program.to_account_info(),
    //     //     close_bet_account,
    //     //     bet_signer,
    //     // );
    //     // close_account(close_bet_cpi_ctx)?;
    // }

    // accumulate cancelled bets
    round.cancelled_bets = round
        .cancelled_bets
        .checked_add(remaining_accounts.len() as u64)
        .ok_or(GoldRushError::Overflow)?;

    // if all bets cancelled
    if round.cancelled_bets >= round.total_bets {
        // close round vault
        let close_vault_account = CloseAccount {
            account: round_vault.to_account_info(),
            destination: ctx.accounts.treasury.to_account_info(),
            authority: round.to_account_info(),
        };
        let vault_seeds = &[
            VAULT_SEED.as_bytes(),
            &round_id.to_le_bytes(),
            &[round.vault_bump],
        ];
        let vault_signer = &[&vault_seeds[..]];
        let close_vault_cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            close_vault_account,
            vault_signer,
        );
        close_account(close_vault_cpi_ctx)?;

        // close round
        let close_round_account = CloseAccount {
            account: round.to_account_info(),
            destination: ctx.accounts.treasury.to_account_info(),
            authority: round.to_account_info(),
        };
        let round_seeds = &[
            ROUND_SEED.as_bytes(),
            &round_id.to_le_bytes(),
            &[round.bump],
        ];
        let round_signer = &[&round_seeds[..]];
        let close_round_cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            close_round_account,
            round_signer,
        );
        close_account(close_round_cpi_ctx)?;
    }

    Ok(())
}
