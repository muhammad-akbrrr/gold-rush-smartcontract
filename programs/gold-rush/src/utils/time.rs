use crate::{constants::*, error::GoldRushError};
use anchor_lang::prelude::*;

pub fn calculate_time_factor(
    market_type: &MarketType,
    time_elapsed: i64,
    min_time_factor_bps: u64,
    max_time_factor_bps: u64,
    duration: i64,
) -> Result<u64> {
    if duration <= 0 {
        return Err(GoldRushError::InvalidDuration.into());
    }

    let factor_bps = match market_type {
        // Linear Decay
        MarketType::GroupBattle => {
            let reduction = (time_elapsed as u128)
                .checked_mul(max_time_factor_bps as u128)
                .ok_or(GoldRushError::Overflow)?
                .checked_div(duration as u128)
                .unwrap_or(0); // Should not fail if duration > 0

            max_time_factor_bps
                .checked_sub(reduction as u64)
                .unwrap_or(min_time_factor_bps) // Default to min if underflow
        }
        // Non-Linear (Quadratic) Decay
        MarketType::SingleAsset => {
            // Formula: (1 - elapsed/duration)^2 * max_bps
            // = ((duration - elapsed)/duration)^2 * max_bps
            // = ((duration - elapsed)^2 * max_bps) / duration^2
            let remaining_time = (duration as u128)
                .checked_sub(time_elapsed as u128)
                .unwrap_or(0);

            let remaining_squared = remaining_time
                .checked_mul(remaining_time)
                .ok_or(GoldRushError::Overflow)?;

            let numerator = remaining_squared
                .checked_mul(max_time_factor_bps as u128)
                .ok_or(GoldRushError::Overflow)?;

            let denominator = (duration as u128)
                .checked_mul(duration as u128)
                .ok_or(GoldRushError::Overflow)?;

            if denominator == 0 {
                return Ok(min_time_factor_bps);
            }

            (numerator.checked_div(denominator).unwrap_or(0)) as u64
        }
    };

    // Ensure result is not lower than the minimum
    Ok(factor_bps.max(min_time_factor_bps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_factor_at_start() {
        // Gold (Non-Linear): At start (elapsed=0) → factor = max = 1.0x
        let factor = calculate_time_factor(&MarketType::SingleAsset, 0, 500, 10_000, 100).unwrap();
        assert_eq!(factor, 10_000);

        // Stock (Linear): At start (elapsed=0) → factor = max = 1.0x
        let factor = calculate_time_factor(&MarketType::GroupBattle, 0, 500, 10_000, 100).unwrap();
        assert_eq!(factor, 10_000);
    }

    #[test]
    fn test_time_factor_halfway_gold() {
        // Gold (Non-Linear): At halfway (elapsed=50/100)
        // Formula: (1 - 0.5)^2 * 10000 = 0.25 * 10000 = 2500
        let factor = calculate_time_factor(&MarketType::SingleAsset, 50, 500, 10_000, 100).unwrap();
        assert_eq!(factor, 2_500);
    }

    #[test]
    fn test_time_factor_halfway_stock() {
        // Stock (Linear): At halfway (elapsed=50/100)
        // Formula: (1 - 0.5) * 10000 = 0.5 * 10000 = 5000
        let factor = calculate_time_factor(&MarketType::GroupBattle, 50, 500, 10_000, 100).unwrap();
        assert_eq!(factor, 5_000);
    }

    #[test]
    fn test_time_factor_at_end() {
        // At the very end, should return min_time_factor
        let factor =
            calculate_time_factor(&MarketType::SingleAsset, 100, 500, 10_000, 100).unwrap();
        assert_eq!(factor, 500); // min_time_factor_bps

        let factor =
            calculate_time_factor(&MarketType::GroupBattle, 100, 500, 10_000, 100).unwrap();
        assert_eq!(factor, 500);
    }

    #[test]
    fn test_time_factor_invalid_duration() {
        // Duration <= 0 should return error
        let result = calculate_time_factor(&MarketType::SingleAsset, 50, 500, 10_000, 0);
        assert!(result.is_err());

        let result = calculate_time_factor(&MarketType::GroupBattle, 50, 500, 10_000, -10);
        assert!(result.is_err());
    }
}
