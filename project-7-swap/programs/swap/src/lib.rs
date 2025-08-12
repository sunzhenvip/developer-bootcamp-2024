// 消除某些编译警告
#![allow(unexpected_cfgs)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("DAehvmx2vZoWCJi7Qo3Y4YF5vrEWHQRJ288kqKwDy5DV");



/**
    允许用户在两种不同的代币之间进行点对点交换
    1. 创建交换报价 (Make Offer)
    用户可以创建一个交换报价，指定：
        1、愿意提供的代币A数量
        2、希望获得的代币B数量
        3、唯一的报价ID
    当用户创建报价时，系统会：
        1、将用户提供的代币转移到托管账户(vault)中
        2、保存报价信息到链上
    2. 接受交换报价 (Take Offer)
        1、其他用户可以接受现有的报价，完成代币交换
        2、接受报价时系统执行：
            将接受者的代币B转移给报价创建者
            将托管账户中的代币A转移给接受者
            关闭托管账户和报价账户
    这个合约为客户提供：
        1、去中心化交换：无需中心化交易所即可进行代币交换
        2、原子性交易：要么完全成功，要么完全失败，确保交易安全
        3、透明定价：所有报价和交换比率都在链上公开
        4、低成本：相比传统DEX，点对点交换减少了中间费用
**/
#[program]
pub mod swap {
    use super::*;

    pub fn make_offer(
        context: Context<MakeOffer>,
        id: u64,
        token_a_offered_amount: u64,
        token_b_wanted_amount: u64,
    ) -> Result<()> {
        // 这两个函数是 make_offer 指令的核心组成部分，按顺序执行以确保代币安全托管和报价信息正确保存。
        // send_offered_tokens_to_vault 负责资产托管，而 save_offer 负责数据持久化，两者缺一不可。
        instructions::make_offer::send_offered_tokens_to_vault(&context, token_a_offered_amount)?;
        instructions::make_offer::save_offer(context, id, token_b_wanted_amount)
    }

    pub fn take_offer(context: Context<TakeOffer>) -> Result<()> {
        // 首先调用 send_wanted_tokens_to_maker 将接受者的代币发送给制造者
        // 然后调用 withdraw_and_close_vault 将金库中的代币提取给接受者并关闭金库
        instructions::take_offer::send_wanted_tokens_to_maker(&context)?;
        instructions::take_offer::withdraw_and_close_vault(context)
    }
}
