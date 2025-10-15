use anchor_lang::prelude::*;

#[event]
pub struct EmergencyUnpaused {
    pub admin: Pubkey,
    pub config: Pubkey,
}
