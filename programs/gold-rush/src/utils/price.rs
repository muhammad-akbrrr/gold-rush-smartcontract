use crate::constants::*;
use anchor_lang::prelude::*;
use pyth_sdk_solana::load_price_feed_from_account_info;

pub fn normalize_price_to_u64(price: i64, expo: i32) -> Result<u64> {
    let scale = ASSET_PRICE_DECIMALS
        .checked_add(expo)
        .ok_or(crate::error::GoldRushError::Overflow)?;
    let v = if scale >= 0 {
        let mul = 10i128
            .checked_pow(scale as u32)
            .ok_or(crate::error::GoldRushError::Overflow)?;
        (price as i128)
            .checked_mul(mul)
            .ok_or(crate::error::GoldRushError::Overflow)?
    } else {
        let exp = scale.unsigned_abs();
        let div = 10i128
            .checked_pow(exp)
            .ok_or(crate::error::GoldRushError::Overflow)?;
        (price as i128)
            .checked_div(div)
            .ok_or(crate::error::GoldRushError::Underflow)?
    };
    if v < 0 {
        return Err(crate::error::GoldRushError::InvalidAssetPrice.into());
    }
    Ok(u64::try_from(v).map_err(|_| crate::error::GoldRushError::Overflow)?)
}

pub fn load_pyth_price_normalized(
    pyth_ai: &AccountInfo,
    now_ts: i64,
    staleness_secs: i64,
) -> Result<u64> {
    let price_feed = load_price_feed_from_account_info(pyth_ai)
        .map_err(|_| crate::error::GoldRushError::PythError)?;
    let price = price_feed
        .get_price_no_older_than(now_ts, staleness_secs as u64)
        .ok_or(crate::error::GoldRushError::PythError)?;
    normalize_price_to_u64(price.price, price.expo)
}
