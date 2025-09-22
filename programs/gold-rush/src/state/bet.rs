use crate::constants::*;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Bet {
    // --- Identify ---
    pub id: u64,        // The unique identifier of the bet.
    pub round: Pubkey,  // The round this bet is associated with.
    pub bettor: Pubkey, // The address of the player placing the bet.

    // --- Bet Info ---
    pub amount: u64,             // The amount of GRT bet.
    pub direction: BetDirection, // The type of bet (Up, Down, PercentageChange).
    pub claimed: bool,           // Whether the reward has been claimed.
    pub weight: u64,             // The weight of the bet (for reward calculation).

    // --- State ---
    pub status: BetStatus, // The status of the bet (Pending, Won, Lost).

    // --- Metadata ---
    pub created_at: i64, // The timestamp when the bet was placed.
    pub bump: u8,        // A bump seed for PDA.
}
