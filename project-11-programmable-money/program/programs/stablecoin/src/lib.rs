use anchor_lang::prelude::*;
use constants::*;
use instructions::*;
use state::*;
mod constants;
mod error;
mod instructions;
mod state;

declare_id!("6DjiD8tQhJ9ZS3WZrwNubfoBRBrqfWacNR3bXBQ7ir91");


/**
是一个基于 Solana 的稳定币协议，实现了以下核心功能：
抵押品存入和稳定币铸造：用户可以存入 SOL 作为抵押品并铸造稳定币
健康因子监控：通过价格预言机监控抵押率，确保系统安全
清算机制：当抵押率过低时，允许清算人清算头寸
赎回机制：用户可以销毁稳定币并取回抵押品

这个项目是教育性质的稳定币协议实现，展示了 DeFi 中的核心概念如超额抵押、价格预言机集成和清算机制。
README.md:3 所有函数都通过调用相应的处理函数来实现具体逻辑，遵循了 Anchor 框架的最佳实践。
系统使用 Pyth Network 作为价格预言机来获取 SOL/USD 价格数据。

initialize_config
    初始化系统配置账户，设置协议的全局参数
    创建配置账户和铸币账户
    设置系统管理员权限
    初始化协议基础参数
update_config
    更新系统配置参数，特别是最小健康因子
    只有系统管理员可以调用
    调整风险参数以维护系统稳定性
deposit_collateral_and_mint
    功能：核心功能函数，允许用户存入 SOL 抵押品并铸造稳定币
    接收用户的 SOL 作为抵押品
    根据当前价格和抵押率铸造相应数量的稳定币
    确保操作后健康因子符合要求
redeem_collateral_and_burn_tokens
    功能：允许用户销毁稳定币并赎回相应的 SOL 抵押品 lib.rs:32-38
    该函数的具体实现逻辑包括：
    更新抵押品账户余额和铸造数量
    检查操作后的健康因子是否符合要求
    销毁指定数量的稳定币
    将相应的 SOL 转回给用户
liquidate
    功能：清算机制，当用户头寸健康因子过低时，允许清算人清算该头寸 lib.rs:40-42
    检查目标头寸是否符合清算条件
    清算人销毁稳定币获得抵押品和清算奖励
    维护系统整体偿付能力
*/

#[program]
pub mod stablecoin {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        process_initialize_config(ctx)
    }

    pub fn update_config(ctx: Context<UpdateConfig>, min_health_factor: u64) -> Result<()> {
        process_update_config(ctx, min_health_factor)
    }

    pub fn deposit_collateral_and_mint(
        ctx: Context<DepositCollateralAndMintTokens>,
        amount_collateral: u64,
        amount_to_mint: u64,
    ) -> Result<()> {
        process_deposit_collateral_and_mint_tokens(ctx, amount_collateral, amount_to_mint)
    }

    pub fn redeem_collateral_and_burn_tokens(
        ctx: Context<RedeemCollateralAndBurnTokens>,
        amount_collateral: u64,
        amount_to_burn: u64,
    ) -> Result<()> {
        process_redeem_collateral_and_burn_tokens(ctx, amount_collateral, amount_to_burn)
    }

    pub fn liquidate(ctx: Context<Liquidate>, amount_to_burn: u64) -> Result<()> {
        process_liquidate(ctx, amount_to_burn)
    }
}
