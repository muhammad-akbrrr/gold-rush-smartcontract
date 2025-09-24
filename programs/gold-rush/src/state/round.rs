use crate::constants::*;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Round {
    // --- Identity ---
    pub id: u64, // Unique identifier for the round (incremental from config.current_round_counter).
    pub start_time: i64, // The timestamp when round is scheduled to start.
    pub end_time: i64, // The timestamp when round is scheduled to end.
    pub vault: Pubkey, // The vault account holding the bets for this round.
    pub vault_bump: u8, // A bump seed for vault PDA.
    pub market_type: MarketType, // The type of market (GoldPrice, StockPrice).

    // --- State ---
    pub status: RoundStatus, // The current status of the round (Scheduled, Active, PendingSettlement, Ended).
    pub total_pool: u64,     // The total amount of GRT bet in this round.
    pub total_bets: u64,     // The total number of bets placed in this round.
    pub total_fee_collected: u64, // The total fees collected for this round.
    pub total_reward_pool: u64, // The total reward pool after deducting fees.
    pub winners_weight: u64, // The total weight of winning bets (for reward calculation). Default to 0 if no winners.
    pub settled_bets: u64,   // Number of bets that have been processed (for incremental settlement)
    #[max_len(MAX_WINNER_GROUP_IDS)]
    pub winner_group_ids: Vec<u64>, // The IDs of the groups that won the round.

    // --- Metadata ---
    pub created_at: i64,         // The timestamp when the round was created.
    pub settled_at: Option<i64>, // The timestamp when the round was settled.
    pub bump: u8,                // A bump seed for PDA.
}
