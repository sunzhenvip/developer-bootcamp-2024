use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{ self, Mint, TokenAccount, TokenInterface, TransferChecked };

declare_id!("GFdLg11UBR8ZeePW43ZyD1gY4z4UQ96LPa22YBgnn4z8");
#[program]
pub mod vesting {
    use super::*;

    pub fn create_vesting_account(
        ctx: Context<CreateVestingAccount>,
        company_name: String
    ) -> Result<()> {
        *ctx.accounts.vesting_account = VestingAccount {
            owner: ctx.accounts.signer.key(),
            mint: ctx.accounts.mint.key(),
            treasury_token_account: ctx.accounts.treasury_token_account.key(),
            company_name,
            treasury_bump: ctx.bumps.treasury_token_account,
            bump: ctx.bumps.vesting_account,
        };

        Ok(())
    }

    pub fn create_employee_vesting(
        ctx: Context<CreateEmployeeAccount>,
        start_time: i64,
        end_time: i64,
        total_amount: i64,
        cliff_time: i64
    ) -> Result<()> {
        *ctx.accounts.employee_account = EmployeeAccount {
            beneficiary: ctx.accounts.beneficiary.key(),
            start_time,
            end_time,
            total_amount,
            total_withdrawn: 0,
            cliff_time,
            vesting_account: ctx.accounts.vesting_account.key(),
            bump: ctx.bumps.employee_account,
        };

        Ok(())
    }

    pub fn claim_tokens(ctx: Context<ClaimTokens>, _company_name: String) -> Result<()> {
        let employee_account = &mut ctx.accounts.employee_account;
        let now = Clock::get()?.unix_timestamp;

        // Check if the current time is before the cliff time
        if now < employee_account.cliff_time {
            return Err(ErrorCode::ClaimNotAvailableYet.into());
        }
        // Calculate the vested amount
        let time_since_start = now.saturating_sub(employee_account.start_time);
        let total_vesting_time = employee_account.end_time.saturating_sub(
            employee_account.start_time
        );
        let vested_amount = if now >= employee_account.end_time {
            employee_account.total_amount
        } else {
            (employee_account.total_amount * time_since_start) / total_vesting_time
        };

        //Calculate the amount that can be withdrawn
        let claimable_amount = vested_amount.saturating_sub(employee_account.total_withdrawn);
        // Check if there is anything left to claim
        if claimable_amount == 0 {
            return Err(ErrorCode::NothingToClaim.into());
        }
        let transfer_cpi_accounts = TransferChecked {
            from: ctx.accounts.treasury_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.employee_token_account.to_account_info(),
            authority: ctx.accounts.treasury_token_account.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let signer_seeds: &[&[&[u8]]] = &[
            &[
                b"vesting_treasury",
                ctx.accounts.vesting_account.company_name.as_ref(),
                &[ctx.accounts.vesting_account.treasury_bump],
            ],
        ];
        let cpi_context = CpiContext::new(cpi_program, transfer_cpi_accounts).with_signer(
            signer_seeds
        );
        let decimals = ctx.accounts.mint.decimals;
        token_interface::transfer_checked(cpi_context, claimable_amount as u64, decimals)?;
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
    #[account(has_one = owner)]
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

#[error_code]
pub enum ErrorCode {
    #[msg("Claiming is not available yet.")]
    ClaimNotAvailableYet,
    #[msg("There is nothing to claim.")]
    NothingToClaim,
}
