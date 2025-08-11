use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct Collateral {
    // 抵押品存款人的钱包地址 - 标识谁拥有这个抵押品账户
    pub depositor: Pubkey,     // depositor wallet address
    // 存款人的SOL抵押品PDA账户地址 - SOL将存入此账户作为抵押品
    pub sol_account: Pubkey,   // depositor pda collateral account (deposit SOL to this account)
    // 存款人的关联代币账户地址 - 稳定币将铸造到此账户
    pub token_account: Pubkey, // depositor ata token account (mint stablecoins to this account)
    // 当前SOL账户的lamport余额 - 用于健康因子计算
    // lamport是SOL的最小单位（1 SOL = 10^9 lamports）
    pub lamport_balance: u64, // current lamport balance of depositor sol_account (for health check calculation)
    // 当前已铸造的稳定币数量 - 用于健康因子计算
    // 以基础单位表示，已调整小数精度
    pub amount_minted: u64, // current amount stablecoins minted, base unit adjusted for decimal precision (for health check calculation)
    // 此抵押品账户PDA的bump种子值 - 用于PDA验证
    pub bump: u8,           // store bump seed for this collateral account PDA
    // SOL账户PDA的bump种子值 - 用于PDA验证
    pub bump_sol_account: u8, // store bump seed for the  sol_account PDA
    // 账户数据是否已初始化的标志 - 防止重复初始化覆盖某些字段
    pub is_initialized: bool, // indicate if account data has already been initialized (for check to prevent overriding certain fields)
}

#[account]
#[derive(InitSpace, Debug)]
pub struct Config {
    // 协议管理员地址 - 拥有修改协议参数的权限
    pub authority: Pubkey,          // authority of the this program config account
    // 稳定币铸造账户地址 - 这是一个PDA，控制稳定币的发行和销毁
    pub mint_account: Pubkey,       // the stablecoin mint address, which is a PDA
    // 清算阈值 - 决定需要多少额外抵押品
    // 例如：150表示需要150%的抵押率，即100美元债务需要150美元抵押品
    pub liquidation_threshold: u64, // determines how much extra collateral is required
    // 清算奖励百分比 - 清算者执行清算时获得的额外lamport奖励
    // 例如：10表示清算者获得10%的额外奖励
    pub liquidation_bonus: u64,     // % bonus lamports to liquidator for liquidating an account
    // 最小健康因子 - 如果低于此值，抵押品账户可被清算
    // 健康因子 = (抵押品价值 * 清算阈值) / 已铸造稳定币价值
    pub min_health_factor: u64, // minimum health factor, if below min then Collateral account can be liquidated
    // 此配置账户PDA的bump种子值 - 用于PDA验证
    pub bump: u8,               // store bump seed for this config account
    // 稳定币铸造账户PDA的bump种子值 - 用于PDA验证
    pub bump_mint_account: u8,  // store bump seed for the stablecoin mint account PDA
}
