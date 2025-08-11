use crate::{Config, SEED_CONFIG_ACCOUNT};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    // 配置账户 - 存储协议全局参数的PDA账户
    // 必须是可变的，因为我们要修改其中的参数
    #[account(
        mut,
        seeds = [SEED_CONFIG_ACCOUNT],
        bump = config_account.bump,
    )]
    pub config_account: Account<'info, Config>,
}
// 修改健康因子以测试清算指令，没有权限检查，任何人都可以调用
// Change health factor to test liquidate instruction, no authority check, anyone can invoke
pub fn process_update_config(ctx: Context<UpdateConfig>, min_health_factor: u64) -> Result<()> {

    // 获取配置账户的可变引用
    let config_account = &mut ctx.accounts.config_account;

    // 更新最小健康因子参数
    // 这个值决定了何时可以清算用户的抵押品
    // 值越高，清算条件越严格
    config_account.min_health_factor = min_health_factor;

    // 打印更新后的配置信息到程序日志，便于调试
    msg!("Update Config Acccount:{:#?}", ctx.accounts.config_account);
    Ok(())
}
