use crate::{constants::*, error::GoldRushError};
use anchor_lang::prelude::*;

pub fn calculate_direction_factor(
    market_type: &MarketType,
    bet_direction: &BetDirection,
    default_direction_factor_bps: u64,
) -> Result<u64> {
    match bet_direction {
        BetDirection::Up | BetDirection::Down => Ok(default_direction_factor_bps),
        BetDirection::PercentageChangeBps(percent) => {
            if *percent == 0 {
                return Ok(default_direction_factor_bps);
            }

            let abs_percent = percent.checked_abs().ok_or(GoldRushError::Overflow)? as u64;

            match market_type {
                MarketType::GoldPrice => {
                    let square = abs_percent
                        .checked_mul(abs_percent)
                        .ok_or(GoldRushError::Overflow)?;

                    let scaled = square.checked_mul(100).ok_or(GoldRushError::Overflow)?;

                    let factor_bps = (HUNDRED_PERCENT_BPS as u64)
                        .checked_add(scaled)
                        .ok_or(GoldRushError::Overflow)?;

                    return Ok(factor_bps);
                }
                MarketType::StockPrice => {
                    let scaled = abs_percent
                        .checked_mul(100)
                        .ok_or(GoldRushError::Overflow)?;

                    let factor_bps = (HUNDRED_PERCENT_BPS as u64)
                        .checked_add(scaled)
                        .ok_or(GoldRushError::Overflow)?;

                    return Ok(factor_bps);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_up_down_direction() {
        let default = 1_000;

        assert_eq!(
            calculate_direction_factor(&MarketType::GoldPrice, &BetDirection::Up, default).unwrap(),
            default
        );
        assert_eq!(
            calculate_direction_factor(&MarketType::StockPrice, &BetDirection::Down, default)
                .unwrap(),
            default
        );
    }

    #[test]
    fn test_percentage_change_zero() {
        let default = 1_000;
        assert_eq!(
            calculate_direction_factor(
                &MarketType::GoldPrice,
                &BetDirection::PercentageChangeBps(0),
                default
            )
            .unwrap(),
            default
        );
    }

    #[test]
    fn test_gold_percentage_change_positive() {
        let default = 1_000;

        let result = calculate_direction_factor(
            &MarketType::GoldPrice,
            &BetDirection::PercentageChangeBps(5),
            default,
        )
        .unwrap();
        assert_eq!(result, HUNDRED_PERCENT_BPS as u64 + 25 * 100);
    }

    #[test]
    fn test_gold_percentage_change_negative() {
        let default = 1_000;

        let result = calculate_direction_factor(
            &MarketType::GoldPrice,
            &BetDirection::PercentageChangeBps(-3),
            default,
        )
        .unwrap();
        assert_eq!(result, HUNDRED_PERCENT_BPS as u64 + 9 * 100)
    }

    #[test]
    fn test_stock_percentage_change_positive() {
        let default = 1_000;

        let result = calculate_direction_factor(
            &MarketType::StockPrice,
            &BetDirection::PercentageChangeBps(4),
            default,
        )
        .unwrap();
        assert_eq!(result, HUNDRED_PERCENT_BPS as u64 + 400);
    }

    #[test]
    fn test_stock_percentage_change_negative() {
        let default = 1_000;

        let result = calculate_direction_factor(
            &MarketType::StockPrice,
            &BetDirection::PercentageChangeBps(-2),
            default,
        )
        .unwrap();
        assert_eq!(result, HUNDRED_PERCENT_BPS as u64 + 200);
    }
}
