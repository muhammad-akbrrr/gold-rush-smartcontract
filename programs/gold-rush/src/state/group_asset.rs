use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct GroupAsset {
    // --- Identity ---
    pub id: u64,
    pub group: Pubkey,

    // --- State ---
    pub symbol: [u8; 8],
    pub total_assets: u64,
    pub total_final_price: u64,
    pub total_growth_rate_bps: i64,
    pub settled_assets: u64,
    pub avg_growth_rate_bps: Option<i64>,

    // --- Metadata ---
    pub created_at: i64,
    pub bump: u8,
}
