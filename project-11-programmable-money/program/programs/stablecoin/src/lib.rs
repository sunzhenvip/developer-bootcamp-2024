// 消除某些编译警告
#![allow(unexpected_cfgs)]
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
project-11-programmable-money和project-10-lending是两个不同类型的DeFi协议，主要区别如下：

核心功能差异
Project 10 (Lending Protocol) 是一个传统的借贷协议，支持多种操作： lib.rs:16-42
存款 (deposit) - 用户存入资产作为抵押品
借款 (borrow) - 用户借出其他资产
还款 (repay) - 偿还借款
提取 (withdraw) - 提取抵押品
清算 (liquidate) - 清算不良头寸

Project 11 (Programmable Money) 是一个稳定币铸造协议，功能更加专注： idlType.ts:483-517
初始化和更新配置
存入抵押品并铸造稳定币
销毁稳定币并赎回抵押品
清算不良头寸
业务模式差异
Lending Protocol 允许用户借入不同于抵押品的资产。例如，用户可以存入USDC作为抵押品，然后借出SOL： borrow.rs:63-76

Programmable Money 只允许用户铸造稳定币。用户存入SOL作为抵押品，只能铸造出协议的原生稳定币，不能借出其他资产。

账户结构差异
Lending Protocol使用更复杂的用户状态管理，跟踪多种资产的存款和借款： state.rs:32-59

而Programmable Money的配置结构更简单，主要关注稳定币铸造的参数： idlType.ts:488-514

风险管理差异
两个协议都使用健康因子和清算机制，但Lending Protocol的清算逻辑更复杂，需要处理多种资产间的转换： liquidate.rs:87-96
Project 10是一个通用的多资产借贷平台，而Project 11是一个专门的稳定币协议。两者都是教育性项目，展示了DeFi协议的不同设计模式。
Lending协议更接近Aave等传统借贷平台，而Programmable Money更类似MakerDAO的稳定币系统。

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
