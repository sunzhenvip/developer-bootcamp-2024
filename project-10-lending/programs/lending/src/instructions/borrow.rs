use std::f32::consts::E;

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ self, Mint, TokenAccount, TokenInterface, TransferChecked };
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};
use crate::constants::{MAXIMUM_AGE, SOL_USD_FEED_ID, USDC_USD_FEED_ID};
use crate::state::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut, 
        seeds = [mint.key().as_ref()],
        bump,
    )]  
    pub bank: Account<'info, Bank>,
    #[account(
        mut, 
        seeds = [b"treasury", mint.key().as_ref()],
        bump, 
    )]  
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut, 
        seeds = [signer.key().as_ref()],
        bump,
    )]  
    pub user_account: Account<'info, User>,
    #[account( 
        init_if_needed, 
        payer = signer,
        associated_token::mint = mint, 
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>, 
    pub price_update: Account<'info, PriceUpdateV2>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// 1. Check if user has enough collateral to borrow
// 2. Warn if borrowing beyond the safe amount but still allow if within the max borrowable amount
// 3. Make a CPI transfer from the bank's token account to the user's token account
// 4. Update the user's borrowed amount and total borrowed value
// 5. Update the bank's total borrows and total borrow shares
// 1. 检查用户是否有足够的抵押品进行借贷
// 2. 如果借贷超过安全金额则发出警告，但如果在最大可借贷金额内仍允许借贷
// 3. 从银行的代币账户向用户的代币账户进行 CPI 转账
// 4. 更新用户的借贷金额和总借贷价值
// 5. 更新银行的总借贷和总借贷份额
// 这个 process_borrow 函数实现了借贷协议的核心借贷逻辑。 process_borrow 函数首先通过
// Pyth 预言机获取实时价格来计算用户的抵押品价值，然后验证借贷金额是否在允许范围内，最后通过 CPI
// 调用将代币从银行转移给用户并更新相关状态。该函数还包含了利息计算功能 calculate_accrued_interest 使用复利公式来计算累积利息。

pub fn process_borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
    // 检查用户是否有足够的抵押品进行借贷
    // Check if user has enough collateral to borrow
    let bank = &mut ctx.accounts.bank;
    let user = &mut ctx.accounts.user_account;

    // 获取价格更新账户的可变引用
    let price_update = &mut ctx.accounts.price_update;

    let total_collateral: u64;

    // 根据要借贷的代币类型计算总抵押品价值
    match ctx.accounts.mint.to_account_info().key() {
        key if key == user.usdc_address => {
            // 如果借贷 USDC，则使用 SOL 作为抵押品
            let sol_feed_id = get_feed_id_from_hex(SOL_USD_FEED_ID)?; 
            let sol_price = price_update.get_price_no_older_than(&Clock::get()?, MAXIMUM_AGE, &sol_feed_id)?;
            // 计算累积利息
            let accrued_interest = calculate_accrued_interest(user.deposited_sol, bank.interest_rate, user.last_updated)?;
            // 计算总抵押品价值（SOL 价格 × SOL 存款数量）
            total_collateral = sol_price.price as u64 * (user.deposited_sol + accrued_interest);
        },
        _ => {
            // 如果借贷 SOL，则使用 USDC 作为抵押品
            let usdc_feed_id = get_feed_id_from_hex(USDC_USD_FEED_ID)?;
            let usdc_price = price_update.get_price_no_older_than(&Clock::get()?, MAXIMUM_AGE, &usdc_feed_id)?;
            // 计算总抵押品价值（USDC 价格 × USDC 存款数量）
            total_collateral = usdc_price.price as u64 * user.deposited_usdc;

        }
    }
    // 计算可借贷金额（总抵押品价值 × 清算阈值）
    let borrowable_amount = total_collateral as u64 *  bank.liquidation_threshold;

    // 检查请求借贷金额是否超过可借贷金额
    if borrowable_amount < amount {
        return Err(ErrorCode::OverBorrowableAmount.into());
    }

    // 设置代币转账的 CPI 账户结构
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.bank_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.bank_token_account.to_account_info(),
    };

    // 获取代币程序引用
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let mint_key = ctx.accounts.mint.key();
    // 设置 PDA 签名种子，用于银行代币账户的授权
    let signer_seeds: &[&[&[u8]]] = &[
        &[
            b"treasury", // 固定种子 "treasury"
            mint_key.as_ref(),// 铸币地址作为种子
            &[ctx.bumps.bank_token_account], // PDA bump 值
        ],
    ];
    // 创建带有签名者的 CPI 上下文
    let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts).with_signer(signer_seeds);
    // 获取代币的小数位数
    let decimals = ctx.accounts.mint.decimals;

    // 执行代币转账，从银行转给用户
    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

    // 如果这是银行的第一笔借贷，初始化借贷数据
    if bank.total_borrowed == 0 {
        bank.total_borrowed = amount;
        bank.total_borrowed_shares = amount;
    }

    // 计算借贷比例和用户份额
    let borrow_ratio = amount.checked_div(bank.total_borrowed).unwrap();
    let users_shares = bank.total_borrowed_shares.checked_mul(borrow_ratio).unwrap();

    // 更新银行的总借贷和总借贷份额
    bank.total_borrowed += amount;
    bank.total_borrowed_shares += users_shares;

    // 根据借贷的代币类型更新用户的借贷余额
    match ctx.accounts.mint.to_account_info().key() {
        key if key == user.usdc_address => {
            // 如果借贷 USDC，更新用户的 USDC 借贷数据
            user.borrowed_usdc += amount;
            user.deposited_usdc_shares += users_shares;
        },
        _ => {
            // 如果借贷 SOL，更新用户的 SOL 借贷数据
            user.borrowed_sol += amount;
            user.deposited_sol_shares += users_shares;
        }
    }

    Ok(())
}

fn calculate_accrued_interest(deposited: u64, interest_rate: u64, last_update: i64) -> Result<u64> {
    let current_time = Clock::get()?.unix_timestamp;
    let time_elapsed = current_time - last_update;
    let new_value = (deposited as f64 * E.powf(interest_rate as f32 * time_elapsed as f32) as f64) as u64;
    Ok(new_value)
}