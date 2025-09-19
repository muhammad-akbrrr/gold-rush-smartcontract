use anchor_lang::prelude::*;

#[error_code]
pub enum GoldRushError {
    // General Program Errors (0x1000 - 0x1999)
    #[msg("Config has already been initialized")]
    AlreadyInitialized = 0x1000,
    
    #[msg("Unauthorized action for this account")]
    Unauthorized = 0x1001,
    
    #[msg("Program is currently paused")]
    ProgramPaused = 0x1002,
    
    #[msg("Program is in emergency pause state")]
    EmergencyPaused = 0x1003,
    
    #[msg("Program is already paused")]
    AlreadyPaused = 0x1004,
    
    #[msg("Program is already active")]
    AlreadyActive = 0x1005,

    // Configuration Errors (0x2000 - 0x2999)
    #[msg("Fee basis points must be less than 10000")]
    InvalidFee = 0x2000,
    
    #[msg("Minimum bet amount must be greater than 0")]
    InvalidMinBetAmount = 0x2001,
    
    #[msg("Keeper authorities list cannot be empty")]
    NoKeeperAuthorities = 0x2002,
    
    #[msg("Keeper is not authorized")]
    UnauthorizedKeeper = 0x2003,

    // Round Management Errors (0x3000 - 0x3999)
    #[msg("Invalid timestamps: start_time must be less than end_time and in the future")]
    InvalidTimestamps = 0x3000,
    
    #[msg("Round with this ID already exists")]
    RoundAlreadyExists = 0x3001,
    
    #[msg("Invalid round status for this action")]
    InvalidRoundStatus = 0x3002,
    
    #[msg("Round is not ready to be activated yet")]
    RoundNotReady = 0x3003,
    
    #[msg("Round is not in Active status")]
    RoundNotActive = 0x3004,
    
    #[msg("Round has ended, no more bets or withdrawals allowed")]
    RoundEnded = 0x3005,
    
    #[msg("Round has not ended yet")]
    RoundNotEnded = 0x3006,
    
    #[msg("Round is not ready for settlement")]
    RoundNotReadyForSettlement = 0x3007,
    
    #[msg("Asset price must be greater than 0")]
    InvalidAssetPrice = 0x3008,

    // Betting Errors (0x4000 - 0x4999)
    #[msg("Bet amount is below minimum required")]
    BetBelowMinimum = 0x4000,
    
    #[msg("Invalid bet status for this action")]
    InvalidBetStatus = 0x4001,
    
    #[msg("Bet did not win, cannot claim reward")]
    BetNotWon = 0x4002,
    
    #[msg("Reward has already been claimed")]
    AlreadyClaimed = 0x4003,
    
    #[msg("No bets have been placed in this round")]
    NoBetsPlaced = 0x4004,

    // Settlement & Claim Errors (0x5000 - 0x5999)
    #[msg("Error retrieving price from oracle")]
    OracleError = 0x5000,
    
    #[msg("Settlement process failed")]
    SettlementFailed = 0x5001,
    
    #[msg("Vault has insufficient balance for this operation")]
    InsufficientVaultBalance = 0x5002,
    
    #[msg("Error calculating reward amount")]
    RewardCalculationError = 0x5003,

    // Account & Token Errors (0x6000 - 0x6999)
    #[msg("Invalid token account")]
    InvalidTokenAccount = 0x6000,
    
    #[msg("Insufficient balance for this bet")]
    InsufficientBalance = 0x6001,
    
    #[msg("Token mint does not match program configuration")]
    InvalidMint = 0x6002,
    
    #[msg("Token transfer operation failed")]
    TokenTransferFailed = 0x6003,
}
