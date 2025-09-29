use crate::constants::*;
use anchor_lang::prelude::*;

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
