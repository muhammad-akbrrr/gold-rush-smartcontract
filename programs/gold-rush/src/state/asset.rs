use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Asset {
    // --- Identity ---
    pub id: u64,
    pub group: Pubkey,
    pub round: Pubkey,

    // --- State ---
    pub symbol: [u8; 8],
    pub start_price: Option<u64>,
    pub final_price: Option<u64>,
    pub growth_rate_bps: Option<i64>,

    // --- Metadata ---
    pub created_at: i64,
    pub settled_at: Option<i64>,
    pub bump: u8,
}
