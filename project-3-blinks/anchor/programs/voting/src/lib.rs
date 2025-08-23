#![allow(clippy::result_large_err)]
// 消除某些编译警告
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("6RMzzoy8iRv9a6ATQbxva3p5GCLFtBukjVN195aBNmQ8");

/**
    这是一个完整的Solana投票智能合约，使用Anchor框架编写。
    程序实现了三个主要功能：创建投票、添加候选人和投票。
    所有账户都使用PDA（Program Derived Address）来确保唯一性和安全性。
    投票和候选人账户通过种子机制关联，确保数据的完整性和可追溯性
**/
#[program] // Anchor宏，标记这是主程序模块
pub mod voting {
    // 投票程序模块
    use super::*; // 导入父模块的所有内容

    pub fn initialize_poll(
        ctx: Context<InitializePoll>, // 初始化投票的函数，接收账户上下文
        poll_id: u64,                 // 投票的唯一标识符
        description: String,          // 投票的描述信息
        poll_start: u64,              // 投票开始时间（Unix时间戳）
        poll_end: u64,                // 投票结束时间（Unix时间戳）
    ) -> Result<()> {
        let poll = &mut ctx.accounts.poll; // 获取可变引用到投票账户
        poll.poll_id = poll_id; // 设置投票ID
        poll.description = description; // 设置投票描述
        poll.poll_start = poll_start; // 设置投票开始时间
        poll.poll_end = poll_end; // 设置投票结束时间
        poll.candidate_amount = 0; // 初始化候选人数量为0
        Ok(()) // 返回成功结果
    }

    pub fn initialize_candidate(
        ctx: Context<InitializeCandidate>, // 初始化候选人的函数
        candidate_name: String,            // 候选人姓名
        _poll_id: u64,                     // 投票ID（前缀下划线表示在函数体中未直接使用）
    ) -> Result<()> {
        let candidate = &mut ctx.accounts.candidate; // 获取可变引用到候选人账户
        let poll = &mut ctx.accounts.poll; // 获取可变引用到投票账户
        poll.candidate_amount += 1; // 增加投票中的候选人数量
        candidate.candidate_name = candidate_name; // 设置候选人姓名
        candidate.candidate_votes = 0; // 初始化候选人票数为0
        Ok(()) // 返回成功结果
    }

    pub fn vote(ctx: Context<Vote>, _candidate_name: String, _poll_id: u64) -> Result<()> {
        // 投票函数，参数前缀下划线表示在函数体中未直接使用
        let candidate = &mut ctx.accounts.candidate; // 获取可变引用到候选人账户
        candidate.candidate_votes += 1; // 为候选人增加一票
        msg!("Voted for candidate: {}", candidate.candidate_name); // 在程序日志中记录投票的候选人
        msg!("Votes: {}", candidate.candidate_votes); // 在程序日志中记录候选人的总票数
        Ok(()) // 返回成功结果
    }
}

#[derive(Accounts)] // Anchor宏，自动生成账户验证代码
#[instruction(candidate_name: String, poll_id: u64)] // 指定指令参数，用于PDA种子生成
pub struct Vote<'info> {
    // 投票操作的账户验证结构体
    pub signer: Signer<'info>, // 交易签名者

    #[account(
        seeds = [poll_id.to_le_bytes().as_ref()], // 使用投票ID作为PDA种子（小端字节序）
        bump // 自动验证PDA的bump种子
    )]
    pub poll: Account<'info, Poll>, // 投票账户（只读）

    #[account(
        mut, // 标记为可变，允许修改票数
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_bytes()], // 使用投票ID和候选人姓名作为PDA种子
        bump // 自动验证PDA的bump种子
    )]
    pub candidate: Account<'info, Candidate>, // 候选人账户（可变）
}

#[derive(Accounts)]
#[instruction(candidate_name: String, poll_id: u64)]
pub struct InitializeCandidate<'info> {
    #[account(mut)] // 标记为可变，因为需要支付账户创建费用
    pub signer: Signer<'info>, // 交易签名者

    #[account(
        mut, // 标记为可变，因为需要更新候选人数量
        seeds = [poll_id.to_le_bytes().as_ref()], // 使用投票ID作为PDA种子
        bump // 自动验证PDA的bump种子
    )]
    pub poll: Account<'info, Poll>, // 投票账户（可变）

    #[account(
        init, // 初始化新的候选人账户
        payer = signer, // 指定账户创建费用的支付者
        space = 8 + Candidate::INIT_SPACE, // 计算账户所需空间：鉴别器(8字节) + 候选人结构体空间
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_bytes()], // 使用投票ID和候选人姓名作为PDA种子
        bump  // 自动查找有效的bump种子
    )]
    pub candidate: Account<'info, Candidate>, // 要创建的候选人账户

    pub system_program: Program<'info, System>, // 系统程序，用于创建账户
}

// 候选人数据结构
#[account] // Anchor宏，标记这是一个账户数据结构体
#[derive(InitSpace)] // 自动计算结构体所需的存储空间
pub struct Candidate {
    #[max_len(32)] // 限制候选人姓名最大长度为32个字符
    pub candidate_name: String, // 候选人姓名
    pub candidate_votes: u64, // 候选人获得的票数
}

// 初始化投票的账户验证结构体
#[derive(Accounts)] // Anchor宏，自动生成账户验证代码
#[instruction(poll_id: u64)] // 指定指令参数
pub struct InitializePoll<'info> {
    #[account(mut)] // 标记为可变，因为需要支付账户创建费用
    pub signer: Signer<'info>, // 交易签名者
    #[account(
        init, // 初始化新的投票账户
        payer = signer, // 指定账户创建费用的支付者
        space = 8 + Poll::INIT_SPACE, // 计算账户所需空间：鉴别器(8字节) + 投票结构体空间
        seeds = [poll_id.to_le_bytes().as_ref()], // 使用投票ID作为PDA种子
        bump // 自动查找有效的bump种子
    )]
    pub poll: Account<'info, Poll>, // 要创建的投票账户

    pub system_program: Program<'info, System>, // 系统程序，用于创建账户
}

// 投票数据结构(表示投票池 一个有多个投票事件)
#[account] // Anchor宏，标记这是一个账户数据结构体
#[derive(InitSpace)] // 自动计算结构体所需的存储空间
pub struct Poll {
    pub poll_id: u64, // 投票的唯一标识符
    #[max_len(280)] // 限制描述最大长度为280个字符（类似Twitter）
    pub description: String, // 投票的描述信息
    pub poll_start: u64, // 投票开始时间（Unix时间戳）
    pub poll_end: u64, // 投票结束时间（Unix时间戳）
    pub candidate_amount: u64, // 投票中的候选人总数
}
