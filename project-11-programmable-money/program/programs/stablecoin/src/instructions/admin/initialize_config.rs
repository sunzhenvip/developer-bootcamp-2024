use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
   Mint, Token2022,
};
use crate::{Config, LIQUIDATION_BONUS, LIQUIDATION_THRESHOLD, MINT_DECIMALS, MIN_HEALTH_FACTOR, SEED_CONFIG_ACCOUNT, SEED_MINT_ACCOUNT};

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    // 权限账户 - 支付初始化费用并成为协议管理员
    #[account(mut)]
    pub authority: Signer<'info>,

    // 配置账户 - 存储协议全局参数的PDA账户
    // 使用 "config" 作为种子生成确定性地址
    #[account(
        init, 
        payer = authority, 
        space = 8 + Config::INIT_SPACE,
        seeds = [SEED_CONFIG_ACCOUNT],
        bump,
    )]
    pub config_account: Account<'info, Config>,
    // 稳定币铸造账户 - 控制稳定币的发行和销毁
    // 使用 "mint" 作为种子生成确定性地址
    #[account(
        init,
        payer = authority,
        seeds = [SEED_MINT_ACCOUNT],
        bump,
        mint::decimals = MINT_DECIMALS,
        mint::authority = mint_account,
        mint::freeze_authority = mint_account,
        mint::token_program = token_program
    )]
    pub mint_account: InterfaceAccount<'info, Mint>,
    // Token2022 程序 - 处理代币相关操作
    pub token_program: Program<'info, Token2022>,
    // 系统程序 - 处理账户创建和租金支付
    pub system_program: Program<'info, System>,
}


/**

这个初始化函数是整个稳定币协议的入口点，只能被调用一次来设置系统的基础参数。
配置中的参数（如清算阈值、奖励比例等）决定了协议的风险管理策略。
该函数创建的配置账户和mint账户都使用PDA（程序派生地址），确保地址的确定性和安全性。


**/

pub fn process_initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
    *ctx.accounts.config_account = Config {
        // 设置协议管理员为当前调用者
        authority: ctx.accounts.authority.key(),
        // 记录稳定币mint账户地址
        mint_account: ctx.accounts.mint_account.key(),
        // 清算阈值 - 决定何时可以清算抵押品（例如：150%）
        liquidation_threshold: LIQUIDATION_THRESHOLD,
        // 清算奖励 - 清算者获得的额外奖励百分比（例如：10%）
        liquidation_bonus: LIQUIDATION_BONUS,
        // 最小健康因子 - 低于此值的仓位可被清算（例如：1.0）
        min_health_factor: MIN_HEALTH_FACTOR,
        // 保存配置账户的bump值，用于后续PDA验证
        bump: ctx.bumps.config_account,
        // 保存mint账户的bump值，用于后续PDA验证
        bump_mint_account:  ctx.bumps.mint_account,
    };
    // 打印配置信息到程序日志，便于调试
    msg!("Initialized Config Acccount:{:#?}", ctx.accounts.config_account);
    Ok(())
}

