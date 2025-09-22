use crate::{constants::*};

pub fn is_bet_winner(
    bet_direction: BetDirection,
    price_change: i64
) -> Option<bool> {
    if price_change == 0 {
        return None;
    }

    match bet_direction {
        BetDirection::Up => Some(price_change > 0),
        BetDirection::Down => Some(price_change < 0),
        BetDirection::PercentageChangeBps(percent) => {
            Some((percent > 0 && price_change > 0)
              || (percent < 0 && price_change < 0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_bet_winner_true() {
        assert_eq!(is_bet_winner(BetDirection::Up, 1), Some(true));
        assert_eq!(is_bet_winner(BetDirection::Down, -1), Some(true));
        assert_eq!(is_bet_winner(BetDirection::PercentageChangeBps(1), 1), Some(true));
        assert_eq!(is_bet_winner(BetDirection::PercentageChangeBps(-1), -1), Some(true));
    }

    #[test]
    fn test_is_bet_winner_false() {
        assert_eq!(is_bet_winner(BetDirection::PercentageChangeBps(0), 1), Some(false));
        assert_eq!(is_bet_winner(BetDirection::PercentageChangeBps(0), -1), Some(false));
        assert_eq!(is_bet_winner(BetDirection::Up, -1), Some(false));
        assert_eq!(is_bet_winner(BetDirection::Down, 1), Some(false));
        assert_eq!(is_bet_winner(BetDirection::PercentageChangeBps(1), -1), Some(false));
        assert_eq!(is_bet_winner(BetDirection::PercentageChangeBps(-1), 1), Some(false));
    }

    #[test]
    fn test_is_bet_winner_none() {
        assert_eq!(is_bet_winner(BetDirection::Up, 0), None);
        assert_eq!(is_bet_winner(BetDirection::Down, 0), None);
        assert_eq!(is_bet_winner(BetDirection::PercentageChangeBps(0), 0), None);
    }
}