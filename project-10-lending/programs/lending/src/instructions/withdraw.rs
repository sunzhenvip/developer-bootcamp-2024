use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ self, Mint, TokenAccount, TokenInterface, TransferChecked };
use crate::state::*;
use crate::error::ErrorCode;

#[derive(Accounts)]
pub struct Withdraw<'info> {
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

// 1. CPI transfer from bank's token account to user's token account
// 2. Calculate new shares to be removed from the bank
// 3. Update user's deposited amount and total collateral value
// 4. Update bank's total deposits and total deposit shares
// 5. Update users health factor ??
// 这个 process_withdraw 方法实现了用户从借贷协议中提取存款的核心逻辑。
// 方法首先验证用户有足够的存款余额，然后通过 CPI 调用将代币从银行的 PDA 账户转移到用户账户，
// 最后更新相关的状态数据。该方法使用份额系统来跟踪用户的存款比例，确保利息分配的准确性。

pub fn process_withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    // 获取用户账户的可变引用
    let user = &mut ctx.accounts.user_account;

    let deposited_value;

    // 检查要提取的代币类型，确定用户的存款余额
    // FIXME: Change from if statement to match statement?? Use PDA deserialization to get the mint address??
    // FIXME: 将 if 语句改为 match 语句？使用 PDA 反序列化获取铸币地址？
    if ctx.accounts.mint.to_account_info().key() == user.usdc_address {
        // 如果是 USDC，获取用户的 USDC 存款余额
        deposited_value = user.deposited_usdc;
    } else {
        // 否则是 SOL，获取用户的 SOL 存款余额
        deposited_value = user.deposited_sol;
    }

    // 检查用户是否有足够的存款余额进行提取
    if amount > deposited_value {
        return Err(ErrorCode::InsufficientFunds.into());
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
            b"treasury",
            mint_key.as_ref(),
            &[ctx.bumps.bank_token_account],
        ],
    ];
    // 创建带有签名者的 CPI 上下文
    let cpi_ctx = CpiContext::new(cpi_program, transfer_cpi_accounts).with_signer(signer_seeds);

    // 获取代币的小数位数
    let decimals = ctx.accounts.mint.decimals;

    // 执行代币转账，从银行转给用户
    token_interface::transfer_checked(cpi_ctx, amount, decimals)?;

    // 获取银行账户的可变引用
    let bank = &mut ctx.accounts.bank;
    // 计算需要移除的份额数量（基于提取金额占总存款的比例）
    let shares_to_remove = (amount as f64 / bank.total_deposits as f64) * bank.total_deposit_shares as f64;

    // 重新获取用户账户的可变引用
    let user = &mut ctx.accounts.user_account;

    // 根据代币类型更新用户的存款余额
    if ctx.accounts.mint.to_account_info().key() == user.usdc_address {
        // 如果是 USDC，减少用户的 USDC 存款
        user.deposited_usdc -= shares_to_remove as u64;
    } else {
        // 如果是 SOL，减少用户的 SOL 存款
        user.deposited_sol -= shares_to_remove as u64;
    }

    // 更新银行的总存款和总份额
    bank.total_deposits -= amount; // 减少银行总存款
    bank.total_deposit_shares -= shares_to_remove as u64; // 减少银行总份额
    
    Ok(())    
}