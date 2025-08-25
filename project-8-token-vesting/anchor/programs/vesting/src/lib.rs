// 消除某些编译警告
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ self, Mint, TokenAccount, TokenInterface, TransferChecked };

declare_id!("GFdLg11UBR8ZeePW43ZyD1gY4z4UQ96LPa22YBgnn4z8");


/**
    如何给客户讲解 这个项目业务
    这是一个 代币归属（Vesting）发放管理系统，主要解决企业在区块链上定期、按比例发放代币给员工的问题。
    企业可以一次性把代币放入合约的金库账户。
    合约会按照约定的时间线（开始时间、结束时间、悬崖期）自动计算员工能领取多少代币。
    员工自己到期后可以随时领取，不需要企业手动发放。
适用场景
    团队代币锁仓发放（如区块链项目创始团队）
    员工/顾问奖励（分期解锁）
    投资人分批释放代币
**/
#[program]
pub mod vesting {
    use super::*;

    /**
        企业初始化（创建公司归属账户）
        企业在系统中创建一个归属账户，并绑定一个代币类型（如 USDC、公司发行的 Token）。
        系统会生成一个金库账户（PDA 控制），由智能合约托管代币。
    **/
    pub fn create_vesting_account(
        ctx: Context<CreateVestingAccount>,
        company_name: String
    ) -> Result<()> {
        // 初始化 vesting_account 结构体，设置所有必要的字段
        *ctx.accounts.vesting_account = VestingAccount {
            owner: ctx.accounts.signer.key(), // 设置账户所有者为当前签名者
            mint: ctx.accounts.mint.key(), // 设置代币铸造地址
            treasury_token_account: ctx.accounts.treasury_token_account.key(), // 设置国库(金库)代币账户地址
            company_name, // 设置公司名称
            treasury_bump: ctx.bumps.treasury_token_account, // 保存国库账户的 bump seed
            bump: ctx.bumps.vesting_account, // 保存 vesting 账户的 bump seed
        };

        Ok(())
    }

    /**
        建立员工归属计划
            企业为每个员工设定：
                1、开始时间（start_time）
                2、结束时间（end_time）
                3、总金额（total_amount）
                4、悬崖期（cliff_time，悬崖期前不能领取）
        这些信息记录在链上，无法篡改。
    **/
    pub fn create_employee_vesting(
        ctx: Context<CreateEmployeeAccount>,
        start_time: i64, // 归属开始时间（Unix 时间戳）
        end_time: i64,   // 归属结束时间（Unix 时间戳）
        total_amount: i64,  // 总归属代币数量
        cliff_time: i64  // 悬崖期时间（在此之前无法提取任何代币）
    ) -> Result<()> {
        *ctx.accounts.employee_account = EmployeeAccount {
            beneficiary: ctx.accounts.beneficiary.key(),   // 设置受益人地址
            start_time,  // 设置归属开始时间
            end_time, // 设置归属结束时间
            total_amount,  // 设置总归属代币数量
            total_withdrawn: 0,  // 初始化已提取数量为 0
            cliff_time,  // 设置悬崖期时间
            vesting_account: ctx.accounts.vesting_account.key(), // 关联到对应的 vesting 账户
            bump: ctx.bumps.employee_account,  // 保存员工账户的 bump seed
        };

        Ok(())
    }

    /**
        员工领取代币
            到了悬崖期之后，员工可以随时调用合约领取代币。
            合约会自动计算可领取额度：
                1、若到期 → 全额解锁
                2、若未到期 → 按比例线性解锁
            领取后，系统会记录已领取总额，防止重复领取。
    **/
    pub fn claim_tokens(ctx: Context<ClaimTokens>, _company_name: String) -> Result<()> {
        // 获取可变的员工账户引用
        let employee_account = &mut ctx.accounts.employee_account;
        // 获取当前区块链时间戳
        let now = Clock::get()?.unix_timestamp;

        // 检查当前时间是否在悬崖期之前，如果是则无法提取
        // Check if the current time is before the cliff time
        if now < employee_account.cliff_time {
            return Err(ErrorCode::ClaimNotAvailableYet.into());
        }


        // 领取代笔释放示意图 (线性释放 下方代码表达的是这个意思)
        // start_time ---------------- cliff_time ---------------- end_time
        //     (0%)       悬崖期      (一次性释放累计的)    (之后按比例线性释放)
        // 计算已归属的代币数量
        // Calculate the vested amount
        let time_since_start = now.saturating_sub(employee_account.start_time);  // 计算从开始时间到现在的时长
        let total_vesting_time = employee_account.end_time.saturating_sub(    // 计算总归属时长
            employee_account.start_time
        );

        // 根据时间比例计算已归属数量，如果超过结束时间则全部归属
        let vested_amount = if now >= employee_account.end_time {
            employee_account.total_amount  // 归属期结束，全部代币已归属
        } else {
            (employee_account.total_amount * time_since_start) / total_vesting_time // 按时间比例计算已归属数量
        };


        // 计算可提取数量（已归属 - 已提取）
        //Calculate the amount that can be withdrawn
        let claimable_amount = vested_amount.saturating_sub(employee_account.total_withdrawn);
        // Check if there is anything left to claim
        // 检查是否有代币可以提取
        if claimable_amount == 0 {
            return Err(ErrorCode::NothingToClaim.into());
        }
        // 准备代币转账的 CPI 调用参数
        let transfer_cpi_accounts = TransferChecked {
            from: ctx.accounts.treasury_token_account.to_account_info(), // 从国库账户转出
            mint: ctx.accounts.mint.to_account_info(),  // 代币铸造账户
            to: ctx.accounts.employee_token_account.to_account_info(),  // 转入员工代币账户
            authority: ctx.accounts.treasury_token_account.to_account_info(),  // 转账权限账户
        };
        let cpi_program = ctx.accounts.token_program.to_account_info(); // 代币程序账户


        // 设置签名种子，用于 PDA 签名
        let signer_seeds: &[&[&[u8]]] = &[
            &[
                b"vesting_treasury",   // 固定种子前缀
                ctx.accounts.vesting_account.company_name.as_ref(), // 公司名称作为种子
                &[ctx.accounts.vesting_account.treasury_bump],    // bump seed
            ],
        ];
        // 创建带签名者的 CPI 上下文
        let cpi_context = CpiContext::new(cpi_program, transfer_cpi_accounts).with_signer(
            signer_seeds
        );
        let decimals = ctx.accounts.mint.decimals;  // 获取代币精度
        // 执行代币转账
        token_interface::transfer_checked(cpi_context, claimable_amount as u64, decimals)?;
        // 更新员工账户的已提取总量
        employee_account.total_withdrawn += claimable_amount;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct CreateVestingAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // 创建 vesting 的公司账户
    #[account(
        init,
        space = 8 + VestingAccount::INIT_SPACE,
        payer = signer,
        seeds = [company_name.as_ref()],
        bump
    )]
    pub vesting_account: Account<'info, VestingAccount>, // 初始化，公司 vesting 存储账户
    pub mint: InterfaceAccount<'info, Mint>, // 要归属的 SPL Token 类型  如 USDC USDT
    // 创建完，它就和任何普通 SPL Token 账户一样，能接收代币转账 企业可以通过钱包往这个地址手动转账
    #[account(
        init,
        token::mint = mint,
        token::authority = treasury_token_account,
        payer = signer,
        seeds = [b"vesting_treasury", company_name.as_bytes()],
        bump
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>, // 由合约管理的金库账户（用于转账）
    pub token_program: Interface<'info, TokenInterface>, // SPL Token 程序
    pub system_program: Program<'info, System>, // 系统程序，用于初始化账户
}

#[derive(Accounts)]
pub struct CreateEmployeeAccount<'info> {
    #[account(mut)]
    pub owner: Signer<'info>, // 公司账户（必须是 vesting_account.owner）
    pub beneficiary: SystemAccount<'info>, // 员工的钱包地址
    #[account(has_one = owner)] // 判断 owner 是否是 vesting_account 的 owner(两个字段名需相同否则不能这么写)
    pub vesting_account: Account<'info, VestingAccount>, // 公司 vesting 信息
    #[account(
        init,
        space = 8 + EmployeeAccount::INIT_SPACE,
        payer = owner,
        seeds = [b"employee_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
        bump
    )]
    pub employee_account: Account<'info, EmployeeAccount>, // 初始化员工 vesting 信息
    pub system_program: Program<'info, System>, // 系统程序
}

#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct ClaimTokens<'info> {
    #[account(mut)]
    pub beneficiary: Signer<'info>, // 员工账户（发起领取）
    #[account(
        mut,
        seeds = [b"employee_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
        bump = employee_account.bump,
        has_one = beneficiary,
        has_one = vesting_account
    )]
    pub employee_account: Account<'info, EmployeeAccount>, // 员工 vesting 数据
    #[account(
        mut,
        seeds = [company_name.as_ref()],
        bump = vesting_account.bump,
        has_one = treasury_token_account,
        has_one = mint
    )]
    pub vesting_account: Account<'info, VestingAccount>, // 公司 vesting 数据
    pub mint: InterfaceAccount<'info, Mint>, // Token 铸币信息
    #[account(mut)]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>, // 金库账户（代币来源）
    #[account(
        init_if_needed,
        payer = beneficiary,
        associated_token::mint = mint,
        associated_token::authority = beneficiary,
        associated_token::token_program = token_program
    )]
    pub employee_token_account: InterfaceAccount<'info, TokenAccount>, // 员工的 ATA（接收代币）
    pub token_program: Interface<'info, TokenInterface>, // Token 程序
    pub associated_token_program: Program<'info, AssociatedToken>, // 用于自动创建 ATA
    pub system_program: Program<'info, System>, // 系统程序
}

#[account]
#[derive(InitSpace, Debug)]
pub struct VestingAccount {
    pub owner: Pubkey, // 创建此 Vesting 的公司账户（签名人）
    pub mint: Pubkey, // 归属计划使用的 SPL Token 的铸币地址
    pub treasury_token_account: Pubkey, // 公司金库账户（该账户持有代币）
    #[max_len(50)]
    pub company_name: String, // 公司名称，用作种子，最多 50 字符
    pub treasury_bump: u8, // 	PDA 金库账户的 bump 值
    pub bump: u8, // 当前 vesting_account PDA 的 bump 值
}

#[account]
#[derive(InitSpace, Debug)]
pub struct EmployeeAccount {
    pub beneficiary: Pubkey, // 员工的钱包地址（代币将转给他）
    pub start_time: i64, // 归属起始时间（Unix 时间戳）
    pub end_time: i64, // 归属结束时间
    pub total_amount: i64, // 总归属代币数量
    pub total_withdrawn: i64, // 员工已领取的代币数量
    pub cliff_time: i64, // 崖期时间（该时间点前不能领取）
    pub vesting_account: Pubkey, // 对应的公司 VestingAccount 地址
    pub bump: u8, // PDA bump
}

/**
这两个错误码在 claim_tokens 函数中被使用：
ClaimNotAvailableYet: 当员工尝试在悬崖期（cliff_time）之前提取代币时触发，确保员工必须等到指定的悬崖期后才能开始提取代币
NothingToClaim: 当计算出的可提取数量为 0 时触发，这种情况发生在员工已经提取了所有已归属的代币，或者当前时间点还没有新的代币归属
**/
#[error_code]
pub enum ErrorCode {
    #[msg("Claiming is not available yet.")]
    ClaimNotAvailableYet, // 当前时间还未到达悬崖期，无法提取代币
    #[msg("There is nothing to claim.")]
    NothingToClaim, // 没有可提取的代币（已全部提取或尚未归属）
}


/***
    “cliff_time + 线性释放” 这种模式一般在哪些业务场景会用到：
    常见业务场景
        员工/团队代币激励（Token Vesting for Team/Advisors）
            背景：团队/顾问持有的代币通常会锁仓，防止他们一上线就全部卖掉砸盘。
            应用：
                start_time = 合约部署时（锁仓开始）
                cliff_time = 6 个月（6 个月之前不释放）
                end_time = 48 个月（之后按月/按区块线性释放）
            效果：
                6 个月 cliff 到达 → 一次性释放前 6 个月应得的代币
                之后 42 个月 → 每个月释放一部分
**/