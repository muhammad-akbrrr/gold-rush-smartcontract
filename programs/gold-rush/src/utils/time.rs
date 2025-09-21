use crate::error::GoldRushError;
use anchor_lang::prelude::*;

pub fn calculate_time_factor(
    time_elapsed: i64,
    min_time_factor_bps: u64,
    max_time_factor_bps: u64,
    duration: i64,
) -> Result<u64> {
    if duration <= 0 {
        return Err(GoldRushError::InvalidDuration.into());
    }

    let elapsed_bps = (time_elapsed as u128)
        .checked_mul(max_time_factor_bps as u128)
        .ok_or(GoldRushError::Overflow)?;

    let reduction = elapsed_bps
        .checked_div(duration as u128)
        .ok_or(GoldRushError::Underflow)?;

    let mut factor = max_time_factor_bps
        .checked_sub(reduction as u64)
        .ok_or(GoldRushError::Underflow)?;

    if factor < min_time_factor_bps {
        factor = min_time_factor_bps;
    }

    Ok(factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_factor_basic() {
        let factor = calculate_time_factor(0, 500, 10_000, 100).unwrap();
        assert_eq!(factor, 10_000);
    }
}
