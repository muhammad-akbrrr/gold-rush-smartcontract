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

            let abs_percent_bps = percent.checked_abs().ok_or(GoldRushError::Overflow)? as u64;

            match market_type {
                MarketType::SingleAsset => {
                    let square = (abs_percent_bps as u128)
                        .checked_mul(abs_percent_bps as u128)
                        .ok_or(GoldRushError::Overflow)?;

                    let bonus_bps = square
                        .checked_div(BPS_SCALING_FACTOR as u128)
                        .ok_or(GoldRushError::Overflow)?;

                    (default_direction_factor_bps as u128)
                        .checked_add(bonus_bps)
                        .ok_or(GoldRushError::Overflow)?
                        .try_into() // Convert u128 back to u64
                        .map_err(|_| GoldRushError::Overflow.into())
                }
                MarketType::GroupBattle => {
                    let result = default_direction_factor_bps
                        .checked_add(abs_percent_bps)
                        .ok_or(GoldRushError::Overflow)?;

                    Ok(result)
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
        let default = HUNDRED_PERCENT_BPS as u64; // 10000 = 100% = 1.0x

        assert_eq!(
            calculate_direction_factor(&MarketType::SingleAsset, &BetDirection::Up, default)
                .unwrap(),
            default
        );
        assert_eq!(
            calculate_direction_factor(&MarketType::GroupBattle, &BetDirection::Down, default)
                .unwrap(),
            default
        );
    }

    #[test]
    fn test_percentage_change_zero() {
        let default = HUNDRED_PERCENT_BPS as u64;
        assert_eq!(
            calculate_direction_factor(
                &MarketType::SingleAsset,
                &BetDirection::PercentageChangeBps(0),
                default
            )
            .unwrap(),
            default
        );
    }

    #[test]
    fn test_gold_percentage_change_positive() {
        let default = HUNDRED_PERCENT_BPS as u64; // 10000

        // Input: 500 bps = 5%
        // Formula: 10000 + (500^2 / 10000) = 10000 + 2500 = 12500 (1.25x)
        let result = calculate_direction_factor(
            &MarketType::SingleAsset,
            &BetDirection::PercentageChangeBps(500), // 5% = 500 bps
            default,
        )
        .unwrap();
        assert_eq!(result, 12_500); // 1.25x
    }

    #[test]
    fn test_gold_percentage_change_negative() {
        let default = HUNDRED_PERCENT_BPS as u64;

        // Input: -300 bps = -3%
        // Formula: 10000 + (300^2 / 10000) = 10000 + 900 = 10900 (1.09x)
        let result = calculate_direction_factor(
            &MarketType::SingleAsset,
            &BetDirection::PercentageChangeBps(-300), // -3% = -300 bps
            default,
        )
        .unwrap();
        assert_eq!(result, 10_900); // 1.09x
    }

    #[test]
    fn test_stock_percentage_change_positive() {
        let default = HUNDRED_PERCENT_BPS as u64;

        // Input: 400 bps = 4%
        // Formula: 10000 + 400 = 10400 (1.04x)
        let result = calculate_direction_factor(
            &MarketType::GroupBattle,
            &BetDirection::PercentageChangeBps(400), // 4% = 400 bps
            default,
        )
        .unwrap();
        assert_eq!(result, 10_400); // 1.04x
    }

    #[test]
    fn test_stock_percentage_change_negative() {
        let default = HUNDRED_PERCENT_BPS as u64;

        // Input: -200 bps = -2%
        // Formula: 10000 + 200 = 10200 (1.02x)
        let result = calculate_direction_factor(
            &MarketType::GroupBattle,
            &BetDirection::PercentageChangeBps(-200), // -2% = -200 bps
            default,
        )
        .unwrap();
        assert_eq!(result, 10_200); // 1.02x
    }
}
