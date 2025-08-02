use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ self, Mint, TokenAccount, TokenInterface, TransferChecked };
use crate::state::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct Repay<'info> {
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
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// Repay function just needs to make a CPI transfer from the user's token account into the bank's token account
// // 还款功能只需要从用户的代币账户向银行的代币账户进行 CPI 转账
pub fn process_repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
    // 获取用户账户的可变引用
    let user = &mut ctx.accounts.user_account;

    let borrowed_asset; 

    // Note: For simplicity, interest fees are not included in this calculation
    // 注意：为了简化，此计算中不包括利息费用

    // 根据代币类型确定用户的借贷余额
    match ctx.accounts.mint.to_account_info().key() {
        key if key == user.usdc_address => {
            // 如果是 USDC，获取用户的 USDC 借贷余额
            borrowed_asset = user.borrowed_usdc;
        },
        _ => {
            // 否则是 SOL，获取用户的 SOL 借贷余额
            borrowed_asset = user.borrowed_sol;
        }
    }

    // 检查还款金额是否超过借贷余额
    if amount > borrowed_asset {
        return Err(ErrorCode::OverRepay.into());
    }

    // 设置代币转账的 CPI 账户结构
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.bank_token_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    // 获取代币程序引用
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts);
    let decimals = ctx.accounts.mint.decimals;

    // 执行代币转账，从用户转给银行
    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

    // Note: The checked_ prefix in Rust is used to perform operations safely by checking for potential 
    // arithmetic overflow or other errors that could occur during the computation. If such an error occurs, these methods
    // return None instead of causing a panic.
    // 注意：checked_ 前缀在 Rust 中用于安全地执行操作，通过检查计算过程中可能发生的
    // 算术溢出或其他错误。如果发生此类错误，这些方法返回 None 而不是引起 panic。

    // 获取银行账户的可变引用
    let bank = &mut ctx.accounts.bank;

    // 计算还款比例和用户份额
    let borrowed_ratio = amount.checked_div(bank.total_borrowed).unwrap();
    let users_shares = bank.total_borrowed_shares.checked_mul(borrowed_ratio).unwrap();

    // 重新获取用户账户的可变引用
    let user = &mut ctx.accounts.user_account;

    // 根据代币类型更新用户的借贷余额
    match ctx.accounts.mint.to_account_info().key() {
        key if key == user.usdc_address => {
            // 如果是 USDC，减少用户的 USDC 借贷数据
            user.borrowed_usdc -= amount;
            user.borrowed_usdc_shares -= users_shares;
        },
        _ => {
            // 如果是 SOL，减少用户的 SOL 借贷数据
            user.borrowed_sol -= amount;
            user.borrowed_sol_shares -= users_shares; 
        }
    }

    // 在这里添加"更新健康因子"功能
    // Add in "update health factor" function here

    // 更新银行的总借贷和总借贷份额
    bank.total_borrowed -= amount; // 减少银行总借贷
    bank.total_borrowed_shares -= users_shares; // 减少银行总借贷份额

    Ok(())
}