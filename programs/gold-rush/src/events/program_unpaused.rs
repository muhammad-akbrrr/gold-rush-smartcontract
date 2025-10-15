use anchor_lang::prelude::*;

#[event]
pub struct ProgramUnpaused {
    pub admin: Pubkey,
    pub config: Pubkey,
}
