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
    PendingSettlement, // Ended but settlement failed, needs retry
    Ended,             // Successfully settled
}

/// Enum for market types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum MarketType {
    GoldPrice,
    StockPrice,
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
}

/// Spaces
pub const DISRIMINATOR_SIZE: u8 = 8;

/// Maths
pub const HUNDRED_PERCENT_BPS: u16 = 10_000;

/// Authority
pub const MAX_KEEPER_AUTHORITIES: usize = 5;
