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
    pub single_asset_feed_id: [u8; 32], // The Pyth feed id for the single asset game.
    pub max_price_update_age_secs: u64, // The maximum age of the price update in seconds.

    // --- Fee Config ---
    pub fee_single_asset_bps: u16, // The fee percentage charged on bets based on Single Asset.
    pub fee_group_battle_bps: u16, // The fee percentage charged on bets based on Group Battle.

    // --- Betting Rules ---
    pub min_bet_amount: u64,         // The minimum bet amount.
    pub bet_cutoff_window_secs: i64, // Window before end_time when betting closes.

    // --- Reward Calculations ---
    pub min_time_factor_bps: u16, // The minimum time factor in basis points.
    pub max_time_factor_bps: u16, // The maximum time factor in basis points.
    pub default_direction_factor_bps: u16, // The default direction factor in basis points.

    // --- Global State ---
    pub status: ProgramStatus, // Overall contract status (Active / Paused / EmergencyPaused)
    pub current_round_counter: u64, // Incremental counter for new round IDs

    // --- Metadata ---
    pub version: u8, // The version of the contract.
    pub bump: u8,    // A bump seed for PDA.
}
