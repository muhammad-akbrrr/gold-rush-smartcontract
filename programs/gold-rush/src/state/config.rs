use crate::constants::*;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    // --- Authorities ---
    pub admin: Pubkey, // The administrator of the contract.
    #[max_len(MAX_KEEPER_AUTHORITIES)]
    pub keeper_authorities: Vec<Pubkey>, // The authority for keeper accounts allowed to keeper operations.

    // --- Token & Treasury ---
    pub token_mint: Pubkey, // The Gold Rush Token (GRT) used for betting.
    pub treasury: Pubkey,   // The address where the fees are sent.

    // --- Fee Config ---
    pub fee_gold_price_bps: u16, // The fee percentage charged on bets based on Gold Price.
    pub fee_stock_price_bps: u16, // The fee percentage charged on bets based on stock price.

    // --- Betting Rules ---
    pub min_bet_amount: u64, // The minimum bet amount.

    // --- Reward Calculations ---
    pub min_time_factor_bps: u64, // The minimum time factor in basis points.
    pub max_time_factor_bps: u64, // The maximum time factor in basis points.
    pub default_direction_factor_bps: u64, // The default direction factor in basis points.

    // --- Global State ---
    pub status: ContractStatus, // Overall contract status (Active / Paused / EmergencyPaused)
    pub current_round_counter: u64, // Incremental counter for new round IDs

    // --- Metadata ---
    pub version: u8, // The version of the contract.
    pub bump: u8,    // A bump seed for PDA.
}
