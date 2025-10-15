use anchor_lang::prelude::*;

#[event]
pub struct EmergencyPaused {
    pub admin: Pubkey,
    pub config: Pubkey,
}
