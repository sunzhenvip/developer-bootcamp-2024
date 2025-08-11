use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    // 健康因子低于最小值错误
    // 当用户的健康因子低于协议设定的最小健康因子时触发
    // 这种情况下用户无法进行某些操作（如铸造更多稳定币）
    #[msg("Below Minimum Health Factor")]
    BelowMinimumHealthFactor,
    // 健康因子高于最小值，无法清算健康账户错误
    // 当尝试清算一个健康因子仍然高于最小值的账户时触发
    // 只有不健康的账户（健康因子低于最小值）才能被清算
    #[msg("Above Minimum Health Factor, Cannot Liquidate Healthy Account")]
    AboveMinimumHealthFactor,
    // 价格不应为负数错误
    // 当从价格预言机获取的价格数据为负数时触发
    // 资产价格应该始终为正数
    #[msg("Price should not be negative")]
    InvalidPrice,
}
