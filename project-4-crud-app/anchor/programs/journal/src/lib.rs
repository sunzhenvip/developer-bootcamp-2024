// 消除某些编译警告
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("94L2mJxVu6ZMmHaGsCHRQ65Kk2mea6aTnwWjSdfSsmBC");

/**
    使用 anchor框架写一个增删改查的示例
**/
#[program]
mod journal {
    use super::*;

    pub fn create_journal_entry(
        ctx: Context<CreateEntry>, // 包含创建日记条目所需的所有账户信息
        title: String,             // 日记条目的标题
        message: String,           // 日记条目的内容
    ) -> Result<()> {
        msg!("Journal Entry Created"); // 在程序日志中记录创建操作
        msg!("Title: {}", title); // 记录标题到日志
        msg!("Message: {}", message); // 记录消息内容到日志

        let journal_entry = &mut ctx.accounts.journal_entry; // 获取可变引用到新创建的日记账户
        journal_entry.owner = ctx.accounts.owner.key(); // 设置日记条目的所有者为交易签名者
        journal_entry.title = title; // 将标题存储到账户状态中
        journal_entry.message = message; // 将消息内容存储到账户状态中
        Ok(())
    }

    pub fn update_journal_entry(
        ctx: Context<UpdateEntry>, // 包含更新日记条目所需的所有账户信息
        title: String,             // 日记条目的标题（用于查找现有条目）
        message: String,           // 新的消息内容
    ) -> Result<()> {
        msg!("Journal Entry Updated"); // 在程序日志中记录更新操作
        msg!("Title: {}", title); // 记录标题到日志
        msg!("Message: {}", message); // 记录新消息内容到日志

        let journal_entry = &mut ctx.accounts.journal_entry; // 获取可变引用到现有的日记账户
        journal_entry.message = message; // 更新消息内容（注意：标题不会被更新）

        Ok(())
    }

    pub fn delete_journal_entry(_ctx: Context<DeleteEntry>, title: String) -> Result<()> {
        msg!("Journal entry titled {} deleted", title); // _ctx 参数前缀下划线表示未使用，但Anchor框架会自动处理账户的关闭和租金回收
        Ok(())
    }
}

#[account] // Anchor宏，标记这是一个账户数据结构体
pub struct JournalEntryState {
    pub owner: Pubkey,   // 日记条目的所有者公钥，32字节
    pub title: String,   // 日记条目的标题，动态长度字符串
    pub message: String, // 日记条目的内容，动态长度字符串
}

#[derive(Accounts)] // Anchor宏，自动生成账户验证代码
#[instruction(title: String, message: String)] // 指定指令参数 用于种子生成
pub struct CreateEntry<'info> {
    #[account(
        init, // 初始化新账户
        seeds = [title.as_bytes(), owner.key().as_ref()], // 使用标题和所有者公钥作为PDA种子
        bump, // 自动查找有效的bump种子
        payer = owner,  // 指定账户创建费用的支付者 （即交易签名者）
        space = 8 + 32 + 4 + title.len() + 4 + message.len() // 计算账户所需空间：鉴别器(8) + 公钥(32) + 字符串长度前缀(4) + 标题长度 + 字符串长度前缀(4) + 消息长度
    )]
    pub journal_entry: Account<'info, JournalEntryState>, // 要创建的日记条目账户
    #[account(mut)] // 标记为可变，因为需要支付租金
    pub owner: Signer<'info>, // 交易签名者，必须是日记条目的所有者
    pub system_program: Program<'info, System>, // 系统程序，用于创建账户
}

#[derive(Accounts)] // Anchor宏，自动生成账户验证代码
#[instruction(title: String, message: String)] // 指定指令参数，用于种子生成和空间计算
pub struct UpdateEntry<'info> {
    #[account(
        mut, // 标记账户为可变，允许修改数据
        seeds = [title.as_bytes(), owner.key().as_ref()], // 使用相同的种子找到现有账户
        bump, // 验证PDA的bump种子
        realloc = 8 + 32 + 4 + title.len() + 4 + message.len(), // 重新分配账户空间以适应新的消息长度
        realloc::payer = owner, // 指定重新分配费用的支付者
        realloc::zero = true, // 将新分配的空间初始化为零
    )]
    pub journal_entry: Account<'info, JournalEntryState>, // 要更新的现有日记条目账户
    #[account(mut)] // 标记为可变，因为可能需要支付额外的租金
    pub owner: Signer<'info>, // 交易签名者，必须是日记条目的所有者
    pub system_program: Program<'info, System>, // 系统程序，用于重新分配账户空间
}

#[derive(Accounts)] // Anchor宏，自动生成账户验证代码
#[instruction(title: String)] // 指定指令参数，只需要标题来找到要删除的账户
pub struct DeleteEntry<'info> {
    #[account(
        mut, // 标记账户为可变，允许关闭账户
        seeds = [title.as_bytes(), owner.key().as_ref()], // 使用相同的种子找到要删除的账户
        bump, // 验证PDA的bump种子
        close= owner, // 关闭账户并将剩余租金退还给所有者
    )]
    pub journal_entry: Account<'info, JournalEntryState>, // 要删除的日记条目账户
    #[account(mut)] // 标记为可变，因为将接收退还的租金
    pub owner: Signer<'info>, // 交易签名者，必须是日记条目的所有者
    pub system_program: Program<'info, System>, // 系统程序，用于关闭账户
}
