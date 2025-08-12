use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{Offer, ANCHOR_DISCRIMINATOR};

use super::transfer_tokens;

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct MakeOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = ANCHOR_DISCRIMINATOR + Offer::INIT_SPACE,
        seeds = [b"offer", maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/**
将用户提供的代币转移到托管账户(vault)中
这个函数负责将报价创建者的代币A安全地转移到由程序控制的托管账户
**/
pub fn send_offered_tokens_to_vault(
    context: &Context<MakeOffer>, // 包含所有必要账户的上下文引用
    token_a_offered_amount: u64,  // 用户愿意提供的代币A数量
) -> Result<()> {
    // 调用通用的代币转移函数，执行安全的代币转移操作
    transfer_tokens(
        &context.accounts.maker_token_account_a, // 源账户：报价创建者的代币A账户
        &context.accounts.vault,                 // 目标账户：程序控制的托管账户
        &token_a_offered_amount,                 // 转移数量：用户指定的代币A数量
        &context.accounts.token_mint_a,          // 代币铸币账户：用于验证代币类型
        &context.accounts.maker,                 // 授权者：报价创建者，拥有转移权限
        &context.accounts.token_program,         // 代币程序：执行实际转移操作的程序
    )
}

/**
保存报价信息到链上的报价账户
这个函数将报价的所有关键信息存储到区块链上，供其他用户查看和接受
**/
pub fn save_offer(
    context: Context<MakeOffer>, // 包含所有必要账户的上下文
    id: u64,                     // 报价的唯一标识符
    token_b_wanted_amount: u64,  // 报价创建者希望获得的代币B数量
) -> Result<()> {
    // 设置报价账户的内部数据结构
    context.accounts.offer.set_inner(Offer {
        id,                                                // 报价ID：用于唯一标识这个报价
        maker: context.accounts.maker.key(),               // 报价创建者：记录是谁创建了这个报价
        token_mint_a: context.accounts.token_mint_a.key(), // 代币A铸币地址：报价者提供的代币类型
        token_mint_b: context.accounts.token_mint_b.key(), // 代币B铸币地址：报价者想要的代币类型
        token_b_wanted_amount,     // 期望获得的代币B数量：交换比率的关键信息
        bump: context.bumps.offer, // PDA bump值：用于重新生成报价账户地址
    });
    Ok(())
}
