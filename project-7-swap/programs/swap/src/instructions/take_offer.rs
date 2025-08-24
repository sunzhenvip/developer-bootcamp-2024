use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::Offer;

use super::transfer_tokens;

#[derive(Accounts)]
pub struct TakeOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    pub token_mint_a: InterfaceAccount<'info, Mint>,

    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = token_mint_a,
        has_one = token_mint_b,
        seeds = [b"offer", maker.key().as_ref(), offer.id.to_le_bytes().as_ref()],
        bump = offer.bump
    )]
    offer: Account<'info, Offer>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
    )]
    vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// 将接受者（taker）的代币发送给报价制造者（maker）
pub fn send_wanted_tokens_to_maker(context: &Context<TakeOffer>) -> Result<()> {
    // 调用通用的代币转账函数，将代币从接受者的账户转移到制造者的账户
    transfer_tokens(
        &context.accounts.taker_token_account_b, // 源账户：接受者的 B 代币账户
        &context.accounts.maker_token_account_b, // 目标账户：制造者的 B 代币账户
        &context.accounts.offer.token_b_wanted_amount, // 转账金额：报价中要求的 B 代币数量
        &context.accounts.token_mint_b,          // B 代币的铸币账户
        &context.accounts.taker,                 // 授权签名者：接受者
        &context.accounts.token_program,         // 代币程序
    )
}

// 从金库中提取代币并关闭金库账户
pub fn withdraw_and_close_vault(context: Context<TakeOffer>) -> Result<()> {
    // 构建 PDA 签名种子，用于授权金库操作
    let seeds = &[
        b"offer",                                              // 固定字符串种子
        context.accounts.maker.to_account_info().key.as_ref(), // 制造者公钥
        &context.accounts.offer.id.to_le_bytes()[..],          // 报价 ID（小端字节序）
        &[context.accounts.offer.bump],                        // PDA bump 种子
    ];
    let signer_seeds = [&seeds[..]];

    // 准备代币转账的账户结构
    let accounts = TransferChecked {
        from: context.accounts.vault.to_account_info(), // 源账户：金库
        to: context.accounts.taker_token_account_a.to_account_info(), // 目标账户：接受者的 A 代币账户
        mint: context.accounts.token_mint_a.to_account_info(),        // A 代币的铸币账户
        authority: context.accounts.offer.to_account_info(),          // 授权账户：报价 PDA
    };
    // CpiContext::new 适合用户自己转账，用户主动发起的转账
    // CpiContext::new_with_signer 需要手动构造 signer_seeds 构造 PDA 的签名(因为 PDA 是由种子和 bump 推导出来的)
    // 适合 合约自己控制的账户转账（比如 vault → 用户，vault → 另一个 vault）。
    // 创建跨程序调用（CPI）上下文，用于调用 SPL Token 程序
    // 创建带签名的跨程序调用上下文
    let cpi_context = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );
    // 执行代币转账：将金库中的所有代币转给接受者
    transfer_checked(
        cpi_context,
        context.accounts.vault.amount, // 转账金额：金库中的全部余额
        context.accounts.token_mint_a.decimals, // 代币精度
    )?;
    // 准备关闭账户的结构
    let accounts = CloseAccount {
        account: context.accounts.vault.to_account_info(),
        destination: context.accounts.taker.to_account_info(),
        authority: context.accounts.offer.to_account_info(),
    };
    // 创建关闭账户的跨程序调用上下文
    let cpi_context = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );
    // 关闭金库账户，将租金返还给接受者
    close_account(cpi_context)
}
