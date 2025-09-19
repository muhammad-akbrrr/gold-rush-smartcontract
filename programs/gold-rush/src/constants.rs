use anchor_lang::prelude::*;

/// PDA Seeds
#[constant]
pub const CONFIG_SEED: &str = "config";
#[constant]
pub const ROUND_SEED: &str = "round";
#[constant]
pub const VAULT_SEED: &str = "vault";
#[constant]
pub const BET_SEED: &str = "bet";

/// Enum for program status flags
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ContractStatus {
    Active,
    Paused,
    EmergencyPaused,
}

/// Enum for market types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum MarketType {
    GoldPrice,
    StockPrice,
}

/// Enum for bet types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum BetSide {
    Up,
    Down,
    PercentageChange(i16),   // e.g., 10 for 0.1%, -25 for -0.25%
}

/// Enum for bet status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum BetStatus {
    Pending,
    Won,
    Lost,
}