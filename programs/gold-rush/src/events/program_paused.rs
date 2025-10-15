use anchor_lang::prelude::*;

#[event]
pub struct ProgramPaused {
    pub admin: Pubkey,
    pub config: Pubkey,
}
