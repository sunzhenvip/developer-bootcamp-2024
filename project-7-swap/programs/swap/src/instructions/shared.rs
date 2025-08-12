use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

// 通用代币转账函数，用于在不同账户之间安全转移代币
pub fn transfer_tokens<'info>(
    from: &InterfaceAccount<'info, TokenAccount>, // 源代币账户（发送方）
    to: &InterfaceAccount<'info, TokenAccount>,   // 目标代币账户（接收方）
    amount: &u64,                                 // 转账金额
    mint: &InterfaceAccount<'info, Mint>,         // 代币铸币账户（用于验证代币类型）
    authority: &Signer<'info>,                    // 授权签名者（必须是源账户的所有者）
    token_program: &Interface<'info, TokenInterface>, // SPL Token 程序接口
) -> Result<()> {
    // 构建转账所需的账户结构体
    let transfer_accounts_options = TransferChecked {
        from: from.to_account_info(), // 将源账户转换为 AccountInfo 类型
        mint: mint.to_account_info(), // 将铸币账户转换为 AccountInfo 类型
        to: to.to_account_info(),     // 将目标账户转换为 AccountInfo 类型
        authority: authority.to_account_info(), // 将授权者转换为 AccountInfo 类型
    };
    // 创建跨程序调用（CPI）上下文，用于调用 SPL Token 程序
    let cpi_context = CpiContext::new(
        token_program.to_account_info(), // SPL Token 程序
        transfer_accounts_options,       // 转账账户结构
    );
    // 执行带检查的代币转账，验证代币类型和精度
    transfer_checked(
        cpi_context,   // CPI 上下文
        *amount,       // 转账金额（解引用）
        mint.decimals, // 代币精度（用于验证转账金额的正确性）
    )
}
