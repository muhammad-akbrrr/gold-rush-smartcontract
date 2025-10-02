use anchor_lang::prelude::*;

/// PDA Seeds
#[constant]
pub const CONFIG_SEED: &str = "config";
#[constant]
pub const ROUND_SEED: &str = "round";
#[constant]
pub const GROUP_ASSET_SEED: &str = "group_asset";
#[constant]
pub const ASSET_SEED: &str = "asset";
#[constant]
pub const VAULT_SEED: &str = "vault";
#[constant]
pub const BET_SEED: &str = "bet";

/// Enum for program status flags
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum ContractStatus {
    Active,
    Paused,
    EmergencyPaused,
}

// Enum for round status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum RoundStatus {
    Scheduled,         // Created but not started yet
    Active,            // Currently accepting bets
    Cancelling,        // Ongoing cancellation
    PendingSettlement, // Ended but settlement failed, needs retry
    Ended,             // Successfully settled
}

/// Enum for market types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum MarketType {
    /// Single-asset market; start/final price read directly from an oracle
    SingleAsset,
    /// Group-battle market; prices captured per-asset and aggregated by groups
    GroupBattle,
}

/// Enum for bet types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum BetDirection {
    Up,
    Down,
    PercentageChangeBps(i16), // e.g., 10 for 0.1%, -25 for -0.25%
}

/// Enum for bet status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum BetStatus {
    Pending,
    Won,
    Lost,
    Draw,
}

/// Spaces
pub const DISRIMINATOR_SIZE: u8 = 8;

/// Maths
pub const HUNDRED_PERCENT_BPS: u16 = 10_000;
pub const BPS_SCALING_FACTOR: u16 = 100;

/// Limits
pub const MAX_KEEPER_AUTHORITIES: usize = 5;
pub const MAX_REMAINING_ACCOUNTS: usize = 20;
pub const MAX_ASSETS_IN_GROUP: usize = 10;
pub const MAX_WINNER_GROUP_IDS: usize = 10;

/// Price
pub const ASSET_PRICE_DECIMALS: i32 = 6;
