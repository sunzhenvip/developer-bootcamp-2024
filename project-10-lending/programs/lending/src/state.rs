use anchor_lang::prelude::*;

/// 借贷协议中的银行账户，用于管理特定代币的资金池。
#[account]
#[derive(InitSpace)]
pub struct Bank {
    /// Authority to make changes to Bank State
    /// 有权限修改银行状态的管理员公钥
    pub authority: Pubkey,
    /// Mint address of the asset
    /// 该银行管理的代币铸币地址（如 SOL 或 USDC）
    pub mint_address: Pubkey,
    /// Current number of tokens in the bank
    /// 银行中当前存款的代币总数量
    pub total_deposits: u64,
    /// Current number of deposit shares in the bank
    /// 银行中存款份额的总数量
    pub total_deposit_shares: u64,
    /// Current number of borrowed tokens in the bank
    /// 银行中当前被借出的代币总数量
    pub total_borrowed: u64,
    /// Current number of borrowed shares in the bank
    /// 银行中借款份额的总数量
    pub total_borrowed_shares: u64,
    /// LTV at which the loan is defined as under collateralized and can be liquidated
    /// 清算阈值，决定何时可以清算抵押不足的头寸
    pub liquidation_threshold: u64,
    /// Bonus percentage of collateral that can be liquidated
    /// 清算奖励百分比
    pub liquidation_bonus: u64,
    /// Percentage of collateral that can be liquidated
    /// 可清算抵押品的百分比
    pub liquidation_close_factor: u64,
    /// Max percentage of collateral that can be borrowed
    /// 最大贷款价值比率
    pub max_ltv: u64,
    /// Last updated timestamp
    /// 最后更新时间戳
    pub last_updated: i64,
    /// 利率
    pub interest_rate: u64,
}

// Challenge: How would you update the user state to save "all_deposited_assets" and "all_borrowed_assets" to accommodate for several asset listings?  
/// 跟踪单个用户在借贷协议中的所有活动和余额
#[account]
#[derive(InitSpace)]
pub struct User {
    /// Pubkey of the user's wallet
    /// 用户钱包的公钥
    pub owner: Pubkey,
    /// User's deposited tokens in the SOL bank
    /// 用户在 SOL 银行中存入的代币数量
    pub deposited_sol: u64,
    /// User's deposited shares in the SOL bank
    /// 用户在 SOL 银行中的存款份额
    pub deposited_sol_shares: u64,
    /// User's borrowed tokens in the SOL bank
    /// 用户从 SOL 银行借入的代币数量
    pub borrowed_sol: u64,
    /// User's borrowed shares in the SOL bank
    /// 用户在 SOL 银行中的借款份额
    pub borrowed_sol_shares: u64, 
    /// User's deposited tokens in the USDC bank
    /// 用户在 USDC 银行中存入的代币数量
    pub deposited_usdc: u64,
    /// User's deposited shares in the USDC bank
    /// 用户在 USDC 银行中的存款份额
    pub deposited_usdc_shares: u64, 
    /// User's borrowed tokens in the USDC bank
    /// 用户从 USDC 银行借入的代币数量
    pub borrowed_usdc: u64,
    /// User's borrowed shares in the USDC bank
    /// 用户在 USDC 银行中的借款份额
    pub borrowed_usdc_shares: u64, 
    /// USDC mint address
    /// USDC 代币的铸币地址
    pub usdc_address: Pubkey,
    /// Current health factor of the user
    /// 用户当前的健康因子
    pub health_factor: u64,
    /// Last updated timestamp
    /// 最后更新时间戳
    pub last_updated: i64,
}

