// 消除某些编译警告
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use instructions::*;

mod state;
mod instructions;
mod error;
mod constants;

declare_id!("CdZeD33fXsAHfZYS8jdxg4qHgXYJwBQ1Bv6GJyETtLST");


/**
    简化的 DeFi 借贷协议，实现了核心的借贷功能。
    1. 管理员功能
        初始化银行 (init_bank)：为特定代币创建银行账户，设置清算阈值和最大贷款价值比率。 admin.rs:46-53
        初始化用户 (init_user)：为用户创建账户来跟踪其存款、借款和份额。 admin.rs:55-64
    2. 用户操作功能
    存款 (deposit)：用户将资产作为抵押品存入银行，使用份额系统跟踪存款。 deposit.rs:47-97
    借款 (borrow)：用户根据抵押品价值借入其他资产，集成 Pyth 预言机获取价格数据。 borrow.rs:54-128
    还款 (repay)：用户偿还借入的资产。 repay.rs:44-105
    提取 (withdraw)：用户提取存入的抵押品。 withdraw.rs:49-102
    3. 清算功能
        清算 (liquidate)：当用户的健康因子低于 1 时，清算人可以清算抵押不足的头寸，获得清算奖励。 liquidate.rs:73-136
    核心概念
        健康因子：计算公式为 (总抵押品 * 清算阈值) / 总借款，当健康因子 < 1 时可被清算。 liquidate.rs:90-91
    份额系统
        使用份额机制跟踪用户存款和借款，实现利息累积而无需更新每个用户余额。 deposit.rs:68-74
    价格预言机集成
        使用 Pyth 预言机获取 SOL 和 USDC 的实时价格数据。 borrow.rs:65-67
    协议定义了两个主要账户结构：
        Bank：存储银行状态，包括总存款、总借款、清算参数等。 state.rs:3-29
        User：跟踪用户的存款、借款、份额和健康因子。 state.rs:32-59

    这是一个教育性质的简化借贷协议，仅支持 SOL 和 USDC 两种资产，包含基本的借贷、清算机制
    但缺少生产环境所需的完整安全措施和复杂的利率模型。协议使用 Anchor 框架开发，集成了 Pyth 预言机进行价格获取。
*/

#[program]
pub mod lending_protocol {

    use super::*;

    pub fn init_bank(ctx: Context<InitBank>, liquidation_threshold: u64, max_ltv: u64) -> Result<()> {
        process_init_bank(ctx, liquidation_threshold, max_ltv)
    }

    pub fn init_user(ctx: Context<InitUser>, usdc_address: Pubkey) -> Result<()> {
        process_init_user(ctx, usdc_address)
    }

    pub fn deposit (ctx: Context<Deposit>, amount: u64) -> Result<()> {
        process_deposit(ctx, amount)
    }

    pub fn withdraw (ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        process_withdraw(ctx, amount)
    }

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        process_borrow(ctx, amount)
    }

    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        process_repay(ctx, amount)
    }

    pub fn liquidate(ctx: Context<Liquidate>) -> Result<()> {
        process_liquidate(ctx)
    }
}

