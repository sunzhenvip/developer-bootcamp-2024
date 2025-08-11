use crate::{
    burn_tokens_internal, calculate_health_factor, error::CustomError, get_lamports_from_usd,
    withdraw_sol_internal, Collateral, Config, SEED_CONFIG_ACCOUNT,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, Token2022, TokenAccount};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

#[derive(Accounts)]
pub struct Liquidate<'info> {
    // 清算者账户 - 执行清算操作的用户钱包，必须支付稳定币来获得抵押品
    #[account(mut)]
    pub liquidator: Signer<'info>,
    // 价格更新账户 - 包含Pyth价格预言机数据，用于计算健康因子和清算价值
    pub price_update: Account<'info, PriceUpdateV2>,
    // 配置账户 - 存储协议全局参数的PDA账户
    // 验证mint_account是否与配置中的一致
    #[account(
        seeds = [SEED_CONFIG_ACCOUNT], // 使用 "config" 种子验证PDA
        bump = config_account.bump, // 使用存储的bump值验证
        has_one = mint_account   // 验证mint_account匹配配置
    )]
    pub config_account: Account<'info, Config>,
    // 被清算用户的抵押品账户 - 跟踪用户抵押品状态的PDA账户
    // 必须是可变的，因为要更新余额和铸造数量
    #[account(
        mut, // 可变，因为要更新账户数据
        has_one = sol_account    // 验证sol_account匹配
    )]
    pub collateral_account: Account<'info, Collateral>,
    // 被清算用户的SOL存储账户 - 实际存储用户SOL抵押品的PDA账户
    // 清算者将从此账户获得SOL抵押品和奖励
    #[account(mut)]
    pub sol_account: SystemAccount<'info>,
    // 稳定币铸造账户 - 控制稳定币发行和销毁的PDA账户
    #[account(mut)]
    pub mint_account: InterfaceAccount<'info, Mint>,
    // 清算者的代币账户 - 清算者用来支付稳定币的关联代币账户
    // 稳定币将从此账户销毁
    #[account(
        mut, // 可变，因为要销毁稳定币
        associated_token::mint = mint_account, // 关联到稳定币mint
        associated_token::authority = liquidator, // 清算者拥有权限
        associated_token::token_program = token_program  // 使用Token2022程序
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    // Token2022 程序 - 处理代币销毁操作
    pub token_program: Program<'info, Token2022>,
    // 系统程序 - 处理SOL转账操作
    pub system_program: Program<'info, System>,
}

/**
这个清算功能是稳定币协议风险管理的核心组件，通过激励机制鼓励第三方清算者维护系统健康。
清算者承担监控市场和执行清算的成本，通过清算奖励获得补偿。
该机制确保即使在市场波动时，协议也能保持稳定和偿付能力。
参考 Cyfrin 的 DeFi 稳定币实现
**/
// https://github.com/Cyfrin/foundry-defi-stablecoin-cu/blob/main/src/DSCEngine.sol#L215
pub fn process_liquidate(ctx: Context<Liquidate>, amount_to_burn: u64) -> Result<()> {
    // 计算当前健康因子，确定是否可以清算
    let health_factor = calculate_health_factor(
        &ctx.accounts.collateral_account,
        &ctx.accounts.config_account,
        &ctx.accounts.price_update,
    )?;

    // 检查健康因子是否低于最小值，只有不健康的账户才能被清算
    // 如果健康因子 >= 最小健康因子，则抛出错误
    require!(
        health_factor < ctx.accounts.config_account.min_health_factor,
        CustomError::AboveMinimumHealthFactor
    );

    // 将要销毁的稳定币数量转换为等值的SOL lamports
    let lamports = get_lamports_from_usd(&amount_to_burn, &ctx.accounts.price_update)?;
    // 计算清算奖励 - 清算者获得的额外SOL奖励
    // 奖励 = SOL价值 * 清算奖励百分比 / 100
    let liquidation_bonus = lamports * ctx.accounts.config_account.liquidation_bonus / 100;
    // 计算总清算金额 = 基础SOL价值 + 清算奖励
    let amount_to_liquidate = lamports + liquidation_bonus;

    // 打印清算信息到程序日志，便于调试和监控
    msg!("*** LIQUIDATION ***");
    msg!("Bonus {}%", ctx.accounts.config_account.liquidation_bonus);
    msg!("Bonus Amount  : {:.9}", liquidation_bonus as f64 / 1e9);
    msg!("SOL Liquidated: {:.9}", amount_to_liquidate as f64 / 1e9);

    // 执行SOL提取 - 将SOL从被清算用户的账户转给清算者
    // 包含基础价值和清算奖励
    withdraw_sol_internal(
        &ctx.accounts.sol_account,
        &ctx.accounts.liquidator.to_account_info(),
        &ctx.accounts.system_program,
        &ctx.accounts.collateral_account.depositor,
        ctx.accounts.collateral_account.bump_sol_account,
        amount_to_liquidate,
    )?;

    // 执行稳定币销毁 - 从清算者的代币账户销毁指定数量的稳定币
    burn_tokens_internal(
        &ctx.accounts.mint_account,
        &ctx.accounts.token_account,
        &ctx.accounts.liquidator,
        &ctx.accounts.token_program,
        amount_to_burn,
    )?;

    // 更新被清算用户的抵押品账户状态
    let collateral_account = &mut ctx.accounts.collateral_account;
    // 更新SOL余额 - 减去被清算的SOL数量
    collateral_account.lamport_balance = ctx.accounts.sol_account.lamports();
    // 更新已铸造的稳定币数量 - 减去被销毁的稳定币数量
    collateral_account.amount_minted -= amount_to_burn;

    // 可选：计算并记录清算后的新健康因子，用于调试
    // Optional, logs new health factor
    calculate_health_factor(
        &ctx.accounts.collateral_account,
        &ctx.accounts.config_account,
        &ctx.accounts.price_update,
    )?;
    Ok(())
}
