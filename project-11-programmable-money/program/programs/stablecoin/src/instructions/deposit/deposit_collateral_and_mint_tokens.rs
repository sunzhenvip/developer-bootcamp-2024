use crate::{
    check_health_factor, deposit_sol_internal, mint_tokens_internal, Collateral, Config,
    SEED_COLLATERAL_ACCOUNT, SEED_CONFIG_ACCOUNT, SEED_SOL_ACCOUNT,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, Token2022, TokenAccount},
};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

#[derive(Accounts)]
pub struct DepositCollateralAndMintTokens<'info> {
    // 存款人账户 - 支付交易费用并签署交易的用户钱包
    #[account(mut)]
    pub depositor: Signer<'info>,

    // 配置账户 - 存储协议全局参数的PDA账户
    // 验证mint_account是否与配置中的一致
    #[account(
        seeds = [SEED_CONFIG_ACCOUNT], // 使用 "config" 种子验证PDA
        bump = config_account.bump, // 使用存储的bump值验证
        has_one = mint_account // 验证mint_account匹配
    )]
    pub config_account: Box<Account<'info, Config>>,
    // 抵押品账户 - 跟踪用户抵押品状态的PDA账户
    // 如果不存在则初始化，存在则复用
    #[account(
        init_if_needed, // 不存在时初始化
        payer = depositor, // 存款人支付租金
        space = 8 + Collateral::INIT_SPACE, // 账户大小
        seeds = [SEED_COLLATERAL_ACCOUNT, depositor.key().as_ref()], // 用户特定的PDA
        bump,  // 自动查找bump值
    )]
    pub collateral_account: Account<'info, Collateral>,
    // SOL存储账户 - 实际存储用户SOL抵押品的PDA账户
    #[account(
        mut, // 可变，因为要接收SOL转账
        seeds = [SEED_SOL_ACCOUNT, depositor.key().as_ref()],  // 用户特定的SOL账户
        bump, // 自动查找bump值
    )]
    pub sol_account: SystemAccount<'info>,
    // 稳定币铸造账户 - 控制稳定币发行的PDA账户
    #[account(mut)]
    pub mint_account: InterfaceAccount<'info, Mint>,
    // 价格更新账户 - 包含Pyth价格预言机数据
    pub price_update: Account<'info, PriceUpdateV2>,
    // 用户代币账户 - 接收铸造稳定币的关联代币账户
    // 如果不存在则自动创建
    #[account(
        init_if_needed, // 不存在时初始化
        payer = depositor, // 存款人支付租金
        associated_token::mint = mint_account,  // 关联到稳定币mint
        associated_token::authority = depositor,  // 存款人拥有权限
        associated_token::token_program = token_program // 使用Token2022程序
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,
    // Token2022 程序 - 处理代币相关操作
    pub token_program: Program<'info, Token2022>,
    // 关联代币程序 - 创建关联代币账户
    pub associated_token_program: Program<'info, AssociatedToken>,
    // 系统程序 - 处理账户创建和SOL转账
    pub system_program: Program<'info, System>,
}

/**
这是稳定币协议的核心功能，允许用户通过存入SOL抵押品来铸造稳定币。
函数首先更新账户状态，然后验证健康因子，最后执行实际的资产转移操作。
健康因子检查确保用户的抵押率满足协议要求，防止系统出现坏账。
**/
// https://github.com/Cyfrin/foundry-defi-stablecoin-cu/blob/main/src/DSCEngine.sol#L140
pub fn process_deposit_collateral_and_mint_tokens(
    ctx: Context<DepositCollateralAndMintTokens>,
    amount_collateral: u64, // 要存入的SOL数量（lamports）
    amount_to_mint: u64,   // 要铸造的稳定币数量
) -> Result<()> {
    // 获取抵押品账户的可变引用
    let collateral_account = &mut ctx.accounts.collateral_account;
    // 更新抵押品账户中的余额记录
    // 计算存入后的总lamport余额
    collateral_account.lamport_balance = ctx.accounts.sol_account.lamports() + amount_collateral;
    // 增加已铸造的稳定币数量
    collateral_account.amount_minted += amount_to_mint;

    // 如果是首次使用，初始化抵押品账户的基本信息
    if !collateral_account.is_initialized {
        collateral_account.is_initialized = true;
        collateral_account.depositor = ctx.accounts.depositor.key();
        collateral_account.sol_account = ctx.accounts.sol_account.key();
        collateral_account.token_account = ctx.accounts.token_account.key();
        collateral_account.bump = ctx.bumps.collateral_account;
        collateral_account.bump_sol_account = ctx.bumps.sol_account;
    }
    // 检查操作后的健康因子是否满足最小要求
    // 如果健康因子过低，交易将失败
    check_health_factor(
        &ctx.accounts.collateral_account,
        &ctx.accounts.config_account,
        &ctx.accounts.price_update,
    )?;
    // 执行SOL转账 - 将SOL从用户钱包转到协议的SOL账户
    deposit_sol_internal(
        &ctx.accounts.depositor,
        &ctx.accounts.sol_account,
        &ctx.accounts.system_program,
        amount_collateral,
    )?;
    // 执行稳定币铸造 - 将新铸造的稳定币发送到用户的代币账户
    mint_tokens_internal(
        &ctx.accounts.mint_account,
        &ctx.accounts.token_account,
        &ctx.accounts.token_program,
        ctx.accounts.config_account.bump_mint_account,
        amount_to_mint,
    )?;
    Ok(())
}
