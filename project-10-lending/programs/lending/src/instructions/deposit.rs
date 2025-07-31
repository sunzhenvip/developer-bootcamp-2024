use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ self, Mint, TokenAccount, TokenInterface, TransferChecked };
use crate::state::*;

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        mut,
        associated_token::mint = mint, 
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>, 
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// 1. CPI transfer from user's token account to bank's token account
// 2. Calculate new shares to be added to the bank
// 3. Update user's deposited amount and total collateral value
// 4. Update bank's total deposits and total deposit shares
// 5. Update users health factor ??
// 函数执行流程总结
// 代币转账：通过 CPI 将用户代币安全转移到银行账户
// 份额计算：根据存款比例计算用户应得的份额
// 用户状态更新：根据代币类型更新用户的存款记录
// 银行状态更新：更新银行的全局存款和份额数据
// 时间戳更新：记录最后操作时间
// 这个函数使用份额系统来跟踪用户存款，允许在不频繁更新每个用户余额的情况下处理利息累积。所有算术运算都使用
// checked_ 前缀来防止溢出错误，体现了 Solana 程序开发的安全最佳实践。
pub fn process_deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    // ========== 步骤 1: 执行代币转账 ==========
    // 设置跨程序调用(CPI)的账户结构，用于将代币从用户账户转移到银行账户
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.bank_token_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    // 获取 SPL Token 程序的引用
    let cpi_program = ctx.accounts.token_program.to_account_info();
    // 创建跨程序调用上下文
    let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts);
    // 获取代币的小数位数，用于精确转账
    let decimals = ctx.accounts.mint.decimals;

    // 执行安全的代币转账，包含小数位数检查
    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

    // ========== 步骤 2: 计算用户应获得的份额 ==========
    // 获取银行账户的可变引用
    // calculate new shares to be added to the bank
    let bank = &mut ctx.accounts.bank;

    // Note: The checked_ prefix in Rust is used to perform operations safely by checking for potential 
    // arithmetic overflow or other errors that could occur during the computation. If such an error occurs, 
    // these methods return None instead of causing a panic.
    // 注释：checked_ 前缀用于安全执行算术运算，检查潜在的溢出错误
    // 如果发生错误，这些方法返回 None 而不是引起 panic

    // 检查是否为银行的首次存款
    if bank.total_deposits == 0 {
        // 首次存款：直接设置总存款和总份额为存款金额
        bank.total_deposits = amount;
        bank.total_deposit_shares = amount;
    }
    // 计算存款比例：当前存款金额 / 银行总存款
    let deposit_ratio = amount.checked_div(bank.total_deposits).unwrap();
    // 计算用户应获得的份额：银行总份额 * 存款比例
    let users_shares = bank.total_deposit_shares.checked_mul(deposit_ratio).unwrap();


    // ========== 步骤 3: 更新用户账户状态 ==========
    // 获取用户账户的可变引用
    let user = &mut ctx.accounts.user_account;

    // 根据代币类型更新用户的存款记录
    match ctx.accounts.mint.to_account_info().key() {
        // 如果是 USDC 代币
        key if key == user.usdc_address => {
            user.deposited_usdc += amount; // 增加 USDC 存款金额
            user.deposited_usdc_shares += users_shares;  // 增加 USDC 存款份额
        },
        // 如果是其他代币（默认为 SOL）
        _ => {
            user.deposited_sol += amount; // 增加 SOL 存款金额
            user.deposited_sol_shares += users_shares; // 增加 SOL 存款份额
        }
    }

    // 注释：上述 match 语句可以轻松添加新分支来支持更多资产类型
    // The above match statement can easily have new branches added when additional assets are added to the protocol

    // ========== 步骤 4: 更新银行全局状态 ==========
    // 更新银行的总存款和总份额
    bank.total_deposits += amount;
    bank.total_deposit_shares += users_shares;

    // ========== 步骤 5: 更新时间戳 ==========
    // 更新用户账户的最后更新时间戳
    user.last_updated = Clock::get()?.unix_timestamp;

    // 返回成功结果
    Ok(())
}