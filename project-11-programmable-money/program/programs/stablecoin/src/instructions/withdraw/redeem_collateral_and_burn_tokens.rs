use crate::{
    burn_tokens_internal, check_health_factor, withdraw_sol_internal, Collateral, Config,
    SEED_COLLATERAL_ACCOUNT, SEED_CONFIG_ACCOUNT,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, Token2022, TokenAccount};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

#[derive(Accounts)]
pub struct RedeemCollateralAndBurnTokens<'info> {
    // 存款人账户 - 签署交易的用户钱包，必须是抵押品的所有者
    #[account(mut)]
    pub depositor: Signer<'info>,

    // 价格更新账户 - 包含Pyth价格预言机数据，用于健康因子计算
    pub price_update: Account<'info, PriceUpdateV2>,
    // 配置账户 - 存储协议全局参数的PDA账户
    // 验证mint_account是否与配置中的一致
    #[account(
        seeds = [SEED_CONFIG_ACCOUNT], // 使用 "config" 种子验证PDA
        bump = config_account.bump,  // 使用存储的bump值验证
        has_one = mint_account  // 验证mint_account匹配配置
    )]
    pub config_account: Account<'info, Config>,
    // 抵押品账户 - 跟踪用户抵押品状态的PDA账户
    // 必须是可变的，因为要更新余额和铸造数量
    #[account(
        mut, // 可变，因为要更新账户数据
        seeds = [SEED_COLLATERAL_ACCOUNT, depositor.key().as_ref()], // 用户特定的PDA
        bump = collateral_account.bump, // 使用存储的bump值验证
        has_one = sol_account,  // 验证sol_account匹配
        has_one = token_account   // 验证token_account匹配
    )]
    pub collateral_account: Account<'info, Collateral>,
    // SOL存储账户 - 实际存储用户SOL抵押品的PDA账户
    // 必须是可变的，因为要从中提取SOL
    #[account(mut)]
    pub sol_account: SystemAccount<'info>,
    // 稳定币铸造账户 - 控制稳定币发行和销毁的PDA账户
    #[account(mut)]
    pub mint_account: InterfaceAccount<'info, Mint>,
    // 用户代币账户 - 存储用户稳定币的关联代币账户
    // 稳定币将从此账户销毁
    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    // Token2022 程序 - 处理代币销毁操作
    pub token_program: Program<'info, Token2022>,
    // 系统程序 - 处理SOL转账操作
    pub system_program: Program<'info, System>,
}

/**
这是稳定币协议的赎回功能，与存入抵押品铸造稳定币的操作相反。 函数首先更新账户状态，
然后验证健康因子确保操作后仍有足够抵押率，最后执行稳定币销毁和SOL提取操作。
健康因子检查是关键的安全措施，防止用户过度提取抵押品导致系统风险。
**/
// https://github.com/Cyfrin/foundry-defi-stablecoin-cu/blob/main/src/DSCEngine.sol#L157
pub fn process_redeem_collateral_and_burn_tokens(
    ctx: Context<RedeemCollateralAndBurnTokens>,
    amount_collateral: u64, // 要赎回的SOL数量（lamports）
    amount_to_burn: u64, // 要销毁的稳定币数量
) -> Result<()> {
    let collateral_account = &mut ctx.accounts.collateral_account;
    // 更新抵押品账户中的余额记录
    // 计算提取后的剩余lamport余额
    collateral_account.lamport_balance = ctx.accounts.sol_account.lamports() - amount_collateral;
    // 减少已铸造的稳定币数量
    collateral_account.amount_minted -= amount_to_burn;
    // 检查操作后的健康因子是否满足最小要求
    // 确保提取抵押品后仍有足够的抵押率
    check_health_factor(
        &ctx.accounts.collateral_account,
        &ctx.accounts.config_account,
        &ctx.accounts.price_update,
    )?;
    // 执行稳定币销毁 - 从用户代币账户销毁指定数量的稳定币
    burn_tokens_internal(
        &ctx.accounts.mint_account,
        &ctx.accounts.token_account,
        &ctx.accounts.depositor,
        &ctx.accounts.token_program,
        amount_to_burn,
    )?;
    // 执行SOL提取 - 将SOL从协议的SOL账户转回用户钱包
    withdraw_sol_internal(
        &ctx.accounts.sol_account,
        &ctx.accounts.depositor.to_account_info(),
        &ctx.accounts.system_program,
        &ctx.accounts.depositor.key(),
        ctx.accounts.collateral_account.bump_sol_account,
        amount_collateral,
    )?;

    Ok(())
}
