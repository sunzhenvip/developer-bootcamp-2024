// 消除某些编译警告
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("5iRyM4LPFo5TZF3SzMqmaFzGhBdNm3mq3v7nVdBcSRpN");

/**
    与之前的project-3-blinks版本相比，增加了时间验证功能。
    vote在投票时会检查当前时间是否在投票的有效时间范围内，并使用自定义错误代码来处理时间相关的错误情况。
    所有账户都使用PDA确保唯一性和安全性。
    https://deepwiki.com/search/project4crudapp-anchorprograms_8ca77df3-bfac-4c81-a667-d0f68815fe00
**/
#[program]
pub mod voting {
    use super::*;

    pub fn initialize_poll(
        ctx: Context<InitializePoll>, // 初始化投票的函数，接收账户上下文
        _poll_id: u64,                // 投票的唯一标识符（前缀下划线表示在函数体中未直接使用）
        start_time: u64,              // 投票开始时间（Unix时间戳）
        end_time: u64,                // 投票结束时间（Unix时间戳）
        name: String,                 // 投票的名称
        description: String,          // 投票的描述信息
    ) -> Result<()> {
        ctx.accounts.poll_account.poll_name = name; // 设置投票名称到账户状态中
        ctx.accounts.poll_account.poll_description = description; // 设置投票描述到账户状态中
        ctx.accounts.poll_account.poll_voting_start = start_time; // 设置投票开始时间到账户状态中
        ctx.accounts.poll_account.poll_voting_end = end_time; // 设置投票结束时间到账户状态中
        Ok(())
    }

    pub fn initialize_candidate(
        ctx: Context<InitializeCandidate>, // 初始化候选人的函数
        _poll_id: u64,                     // 投票ID（前缀下划线表示在函数体中未直接使用）
        candidate: String,                 // 候选人姓名，返回结果类型
    ) -> Result<()> {
        ctx.accounts.candidate_account.candidate_name = candidate; // 设置候选人姓名到账户状态中
        ctx.accounts.poll_account.poll_option_index += 1; // 增加投票中的候选人选项索引
        Ok(())
    }

    pub fn vote(
        ctx: Context<Vote>, // 投票函数，参数前缀下划线表示在函数体中未直接使用
        _poll_id: u64,      // 前缀下划线表示在函数体中未直接使用
        _candidate: String, // 前缀下划线表示在函数体中未直接使用
    ) -> Result<()> {
        let candidate_account = &mut ctx.accounts.candidate_account; // 获取可变引用到候选人账户
        let current_time = Clock::get()?.unix_timestamp; // 获取当前系统时间戳

        if current_time > (ctx.accounts.poll_account.poll_voting_end as i64) {
            // 检查当前时间是否超过投票结束时间
            return Err(ErrorCode::VotingEnded.into()); // 如果投票已结束，返回错误
        }

        if current_time <= (ctx.accounts.poll_account.poll_voting_start as i64) {
            // 检查当前时间是否在投票开始时间之前
            return Err(ErrorCode::VotingNotStarted.into()); // 如果投票尚未开始，返回错误
        }

        candidate_account.candidate_votes += 1; // 为候选人增加一票

        Ok(())
    }
}

// 初始化投票的账户验证结构体
#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitializePoll<'info> {
    #[account(mut)] // 标记为可变，因为需要支付账户创建费用
    pub signer: Signer<'info>, // 交易签名者

    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + PollAccount::INIT_SPACE,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_account: Account<'info, PollAccount>, // 要创建的投票账户

    pub system_program: Program<'info, System>, // 系统程序，用于创建账户
}

// 初始化候选人的账户验证结构体
#[derive(Accounts)]
#[instruction(poll_id: u64, candidate: String)]
pub struct InitializeCandidate<'info> {
    #[account(mut)] // 标记为可变，因为需要支付账户创建费用
    pub signer: Signer<'info>, // 交易签名者

    pub poll_account: Account<'info, PollAccount>, // 投票账户（只读）

    #[account(
        init,
        payer = signer,
        space = 8 + CandidateAccount::INIT_SPACE,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
        bump
    )]
    pub candidate_account: Account<'info, CandidateAccount>, // 要创建的候选人账户

    pub system_program: Program<'info, System>, // 系统程序，用于创建账户
}

// 投票操作的账户验证结构体
#[derive(Accounts)]
#[instruction(poll_id: u64, candidate: String)]
pub struct Vote<'info> {
    #[account(mut)] // 标记为可变，虽然在此结构体中签名者账户本身不会被修改
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"poll".as_ref(), poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll_account: Account<'info, PollAccount>, // 投票账户（可变）

    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
        bump)]
    pub candidate_account: Account<'info, CandidateAccount>, // 候选人账户（可变）
}

// 候选人账户数据结构
#[account]
#[derive(InitSpace)]
pub struct CandidateAccount {
    #[max_len(32)] // 限制候选人姓名最大长度为32个字符
    pub candidate_name: String, // 候选人姓名
    pub candidate_votes: u64, // 候选人获得的票数
}

// 投票账户数据结构
#[account]
#[derive(InitSpace)]
pub struct PollAccount {
    #[max_len(32)] // 限制投票名称最大长度为32个字符
    pub poll_name: String, // 投票名称
    #[max_len(280)] // 限制投票描述最大长度为280个字符
    pub poll_description: String, // 投票描述信息
    pub poll_voting_start: u64, // 投票开始时间（Unix时间戳）
    pub poll_voting_end: u64,   // 投票结束时间（Unix时间戳）
    pub poll_option_index: u64, // 投票选项索引（候选人数量计数器）
}

// 错误代码枚举
#[error_code]
pub enum ErrorCode {
    #[msg("Voting has not started yet")] // 错误消息：投票尚未开始
    VotingNotStarted, // 投票尚未开始错误
    #[msg("Voting has ended")] // 错误消息：投票已结束
    VotingEnded, // 投票已结束错误
}
