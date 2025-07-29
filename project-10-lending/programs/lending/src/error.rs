use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    /// 超过最大贷款价值比率
    /// 当借款金额超过了银行设置的最大贷款价值比率时触发
    /// 用于防止用户借款过多导致风险过高
    #[msg("Borrowed amount exceeds the maximum LTV.")]
    OverLTV,
    /// 抵押不足
    /// 当借款金额导致贷款抵押不足时触发 保护协议免受抵押品价值不足的风险
    #[msg("Borrowed amount results in an under collateralized loan.")]
    UnderCollateralized,
    /// 资金不足
    /// 当用户尝试提取超过其存款余额的资金时触发
    /// 在提取操作中使用，确保用户不能提取超过其拥有的资产
    #[msg("Insufficient funds to withdraw.")]
    InsufficientFunds,
    /// 超额还款
    /// 当用户尝试偿还超过其借款金额的资产时触发
    /// 防止用户还款金额超过实际借款
    #[msg("Attempting to repay more than borrowed.")]
    OverRepay,
    /// 超过可借款金额
    /// 当用户尝试借入超过其抵押品允许的金额时触发
    /// 基于抵押品价值和清算阈值计算可借款上限
    #[msg("Attempting to borrow more than allowed.")]
    OverBorrowableAmount,
    /// 未抵押不足
    /// 当尝试清算健康因子大于等于1的头寸时触发
    /// 确保只有真正抵押不足的头寸才能被清算
    #[msg("User is not undercollateralized.")]
    NotUndercollateralized
}