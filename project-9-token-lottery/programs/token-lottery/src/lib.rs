// 消除某些编译警告
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface}
};
use switchboard_on_demand::accounts::RandomnessAccountData;
use anchor_spl::metadata::{
    Metadata,
    MetadataAccount,
    CreateMetadataAccountsV3,
    CreateMasterEditionV3,
    SignMetadata,
    SetAndVerifySizedCollectionItem,
    create_master_edition_v3,
    create_metadata_accounts_v3,
    sign_metadata,
    set_and_verify_sized_collection_item,
};
use mpl_token_metadata::types::{
    CollectionDetails,
    Creator,
    DataV2,
};


declare_id!("7FzPbWJ1eMVTT8YhhzvgJ1H8Yo2AZ76L7xn7ddGA5uyz");

#[constant]
pub const NAME: &str = "Token Lottery Ticket #";
#[constant]
pub const URI: &str = "Token Lottery";
#[constant]
pub const SYMBOL: &str = "TICKET";

/**
这个合约主要实现了一个基于 Solana NFT 的链上抽奖系统
这个合约让商家或项目方可以在 Solana 链上举办一次完整的抽奖活动，用户购买 NFT 门票参与，系统在活动结束后自动、透明地抽取并发放奖金。
业务价值
    链上透明：所有购票、抽奖、奖金发放过程都在链上可查，防止暗箱操作。
    防伪防篡改：门票 NFT 与抽奖集合绑定，且元数据和验证状态由链上程序控制，无法伪造。
    防作弊：使用 Switchboard 的链上随机数，确保抽奖结果公平。
    一次性活动管理：可以按期配置，每期都有独立的集合 NFT 与门票 NFT，方便管理多期抽奖。
程序基于 Solana 区块链，使用 Anchor 框架实现了一个完整的彩票系统，主要功能如下：
1. 彩票初始化（Initialize）
    设置彩票参数：
        1、彩票开始时间（lottery_start）和结束时间（lottery_end）
        2、每张彩票的价格（price）
        3、彩票池初始金额（lottery_pot_amount）
        4、彩票管理者（authority）
    创建 NFT 集合（Collection NFT）：
        1、使用 Metaplex Token Metadata 标准创建 NFT 集合
        2、设置集合名称（NAME）、符号（SYMBOL）和 URI（URI）
        3、生成 Master Edition（确保 NFT 唯一性）
2. 购买彩票（Buy Ticket）
    用户支付 SOL 购买彩票：
        1、检查彩票是否在开放时间内（lottery_start ≤ 当前时间 ≤ lottery_end）
        2、支付 SOL 到彩票池（lottery_pot_amount 增加）
    生成彩票 NFT：
        1、每张彩票是一个 NFT，名称格式为 Token Lottery Ticket #X（X 是序号）
        2、彩票 NFT 属于之前创建的 Collection NFT（确保可验证）
        3、使用 Metaplex 标准 生成 NFT 元数据和 Master Edition
3. 提交随机数（Commit Randomness）
    使用 Switchboard 随机数服务：
        1、通过 RandomnessAccountData 获取链上随机数
        2、确保随机数是 最新且未被使用 的（seed_slot == current_slot - 1）
        3、存储随机数账户地址（randomness_account）
4. 选择中奖者（Choose Winner）
    验证彩票是否结束：
        1、检查当前时间是否超过 lottery_end
        2、确保 尚未选出中奖者（winner_chosen == false）
    计算中奖者：
        1、使用 Switchboard 随机数 对彩票总数取模（random_value % ticket_num）
        2、记录中奖者编号（winner）并标记已开奖（winner_chosen = true）
5. 领取奖金（Claim Prize）
    验证中奖者：
        1、检查是否已开奖（winner_chosen == true）
        2、检查用户持有的 NFT 是否属于 Collection NFT（metadata.collection.verified）
        3、检查 NFT 名称是否匹配中奖编号（Token Lottery Ticket #X）
    发放奖金：
        1、将彩票池的 SOL 转给中奖者
        2、清空彩票池（lottery_pot_amount = 0）
6. 错误处理（Error Handling）
    定义多种错误情况，如：
        LotteryNotOpen（彩票未开放）
        LotteryNotCompleted（彩票未结束）
        WinnerNotChosen（尚未开奖）
        NotVerifiedTicket（NFT 未验证）
        IncorrectTicket（非中奖 NFT）
总结
    1、你的程序实现了一个 去中心化彩票系统，主要流程包括：
    2、初始化彩票参数
    3、用户购买 NFT 彩票
    4、链上随机数开奖
    5、中奖者领取奖金
优点：
✅ 使用 NFT 作为彩票，确保唯一性和可验证性
✅ 结合 Switchboard 随机数，保证公平性
✅ 完整的状态管理（开奖、奖金发放）
可能的改进：
🔹 支持 多期彩票（目前只能运行一期）
🔹 增加 手续费机制（部分 SOL 作为平台收益）
🔹 优化 随机数获取方式（如改用 Chainlink VRF）
整体来说，代码结构清晰，功能完整，是一个不错的 Solana 彩票系统实现！
**/

#[program]
pub mod token_lottery {

    use super::*;
    // 初始化抽奖配置（抽奖时间段、票价、初始状态）
    pub fn initialize_config(ctx: Context<InitializeConifg>, start: u64, end: u64, price: u64) -> Result<()> {
        ctx.accounts.token_lottery.bump = ctx.bumps.token_lottery;
        ctx.accounts.token_lottery.lottery_start = start;
        ctx.accounts.token_lottery.lottery_end = end;
        ctx.accounts.token_lottery.price = price;
        ctx.accounts.token_lottery.authority = ctx.accounts.payer.key();
        ctx.accounts.token_lottery.randomness_account = Pubkey::default();

        ctx.accounts.token_lottery.ticket_num = 0;
        ctx.accounts.token_lottery.winner_chosen = false;
        Ok(())
    }
    // 创建抽奖 NFT 集合（Collection Mint + Metadata）
    /**
        构造 signer_seeds	后续 PDA 操作的签名凭证
        mint_to	铸造 1 个 Collection NFT
        create_metadata_accounts_v3	创建 NFT 元数据信息
        create_master_edition_v3	创建主版本 NFT（Master Edition）
        sign_metadata	使 Collection NFT 变成“已签名集合”可被子 NFT 验证关联
        这是创建的一个 Collection NFT 集合 类型的 NFT
        initialize_lottery
            这个函数是 管理员调用 的，用来初始化整个抽奖活动。
            它主要做了几件事：
        1、创建集合 NFT（Collection Mint）
            生成一个 collection_mint，这是集合的 Mint 地址，相当于 NFT 集合的「根」。
            并且给集合 mint 铸造 1 个 token（存在 collection_token_account）。
        2、创建集合的 Metadata
            通过 create_metadata_accounts_v3，把集合 NFT 的名字、符号、URI 等信息写入链上。
            设置 CollectionDetails::V1 { size: 0 }，明确这是一个 集合 NFT。
        3、创建集合的 Master Edition
            调用 create_master_edition_v3，说明这是个 不可再分割的集合 NFT（master edition）。
        4、签名确认集合 NFT
            通过 sign_metadata 给集合 NFT 签名，确认它的合法性。
        总结：initialize_lottery 的结果就是创建了一个 集合 NFT，所有用户之后买的票（Ticket NFT）都会属于这个集合。
    **/
    pub fn initialize_lottery(ctx: Context<InitializeLottery>) -> Result<()> {
        // 构造 PDA signer 的 seeds，用于后续 CPI 调用中授权 PDA 签名
        // Create Collection Mint
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"collection_mint".as_ref(),
            &[ctx.bumps.collection_mint],
        ]];
        // Step 1: 使用 PDA（collection_mint）铸造 1 个 token（即 Collection NFT）
        msg!("Creating mint accounts");
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.collection_mint.to_account_info(),  // Collection NFT 的 mint 地址
                    to: ctx.accounts.collection_token_account.to_account_info(), // 接收铸造出的 NFT token 的账户
                    authority: ctx.accounts.collection_mint.to_account_info(), // 使用 PDA 作为 mint authority
                },
                signer_seeds, // 声明 PDA 签名权
            ),
            1,  // 铸造一个 token
        )?;
        // Step 2: 创建 collection NFT 的 metadata 信息（链上注册 NFT 基本信息）
        msg!("Creating metadata accounts");
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata.to_account_info(), // 存储 metadata 数据的 PDA 账户
                    mint: ctx.accounts.collection_mint.to_account_info(), // 对应的 mint（NFT）
                    mint_authority: ctx.accounts.collection_mint.to_account_info(), // use pda mint address as mint authority
                    update_authority: ctx.accounts.collection_mint.to_account_info(), // use pda mint as update authority
                    payer: ctx.accounts.payer.to_account_info(), // 支付 rent 的账户
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            DataV2 {
                name: NAME.to_string(), // NFT 名称，例如 "Token Lottery Ticket #"
                symbol: SYMBOL.to_string(), // 代号，例如 "TICKET"
                uri: URI.to_string(),  // 链接，通常是元数据托管在 IPFS 或 Arweave 上
                seller_fee_basis_points: 0, // 二级市场分成，这里设为 0（无佣金）
                creators: Some(vec![Creator {
                    address: ctx.accounts.collection_mint.key(), // 设置 PDA 为创建者
                    verified: false,                             // 初始未签名
                    share: 100,                                  // 占有 100% 权限
                }]),
                collection: None,
                uses: None,
            },
            true,  // is_mutable: 允许更新
            true, // update_authority_is_signer: 是签名者
            Some(CollectionDetails::V1 { size: 0 }), // set as collection nft  // 标记这是一个 collection 类型 NFT，初始 size = 0
        )?;
        // Step 3: 创建 collection 的 Master Edition（表示该 NFT 是一个主版本）
        msg!("Creating Master edition accounts");
        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    payer: ctx.accounts.payer.to_account_info(), // 支付 rent 的账户
                    mint: ctx.accounts.collection_mint.to_account_info(), // mint 地址
                    edition: ctx.accounts.master_edition.to_account_info(), // master edition PDA 地址
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            Some(0),
        )?;
        // Step 4: 用 collection mint PDA 对 metadata 签名，确立为 verified creator
        msg!("verifying collection");
        sign_metadata(CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            SignMetadata {
                creator: ctx.accounts.collection_mint.to_account_info(), // 签名者（PDA）
                metadata: ctx.accounts.metadata.to_account_info(),       // 要签名的 metadata
            },
            &signer_seeds,
        ))?;


        Ok(())
    }

    // 用户购票，铸造 Ticket NFT 并加入集合
    /**
        检查抽奖是否开放（按 slot 时间）
        从买票人向奖池账户转账购票费用
        使用合约 PDA 铸造一张 NFT 票
        为 NFT 票创建元数据和 master edition
        将 NFT 设置进集合
        更新票号，供下次使用
        buy_ticket 这个函数是 用户调用 的，用来购买抽奖票 NFT。 它做的事情是：
        1、用户支付门票价
            通过 system_program::transfer 把 SOL 转到 token_lottery 账户里，形成奖池。
        2、创建票据 NFT（Ticket Mint）
            为每个用户买的票生成一个新的 ticket_mint，即 Ticket NFT 的 mint 地址。
            这个票 NFT 的名字是 "Token Lottery Ticket #<编号>"。
        3、创建票据的 Metadata + Master Edition
            每个 Ticket NFT 都有自己的 Metadata（名字、符号、URI 等）。
            也会创建 Master Edition。
        4、把 Ticket NFT 验证为集合的一部分
            调用 set_and_verify_sized_collection_item，把刚创建的 Ticket NFT 加入到之前 initialize_lottery 创建的集合里。
            这样 Ticket NFT 就「挂靠」在集合 NFT 下面了。
        👉 总结：每次用户调用 buy_ticket，都会生成一个新的 Ticket NFT，它属于 initialize_lottery 创建的集合。
        5、你的理解可以这样归纳：
            管理员端：initialize_lottery 创建 集合 NFT（相当于标签/父类）。
            用户端：buy_ticket 购买一个 票据 NFT，并且自动挂到集合 NFT 下面。
    **/
    pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
        // 获取当前区块时间（slot）
        let clock = Clock::get()?;
        // 根据票号生成当前票 NFT 的名称，如 "Ticket0", "Ticket1" 等
        let ticket_name = NAME.to_owned() + ctx.accounts.token_lottery.ticket_num.to_string().as_str();
        // 检查当前时间是否处于抽奖开放时间内（slot 在开始和结束之间）
        if clock.slot < ctx.accounts.token_lottery.lottery_start || 
                clock.slot > ctx.accounts.token_lottery.lottery_end {
            return Err(ErrorCode::LotteryNotOpen.into());
        }

        // 转账购票费用：将 SOL 从参与者转入奖池账户（token_lottery）
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.token_lottery.to_account_info(),
                },
            ),
            ctx.accounts.token_lottery.price,
        )?;
        // 累加奖池金额
        ctx.accounts.token_lottery.lottery_pot_amount += ctx.accounts.token_lottery.price;
        // 构造 signer PDA 用于授权 mint 权限（collection_mint 是该合约控制的 mint PDA）
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"collection_mint".as_ref(),
            &[ctx.bumps.collection_mint],
        ]];
        // 使用合约 PDA authority（collection_mint）铸造 1 张票（1 个 NFT token）
        // Mint Ticket
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.ticket_mint.to_account_info(), // 新票的 mint 地址
                    to: ctx.accounts.destination.to_account_info(),  // 买票者的 token account
                    authority: ctx.accounts.collection_mint.to_account_info(), // 授权者是 PDA
                },
                signer_seeds,
            ),
            1, // NFT 只能铸造一个单位
        )?;
        // 创建该票的元数据（Metadata），包括名称、symbol、uri 等信息
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint: ctx.accounts.ticket_mint.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(), // 创建 metadata 的费用由买票者承担
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            DataV2 {
                name: ticket_name,          // 票名称，如 Ticket0
                symbol: SYMBOL.to_string(), // NFT 的 symbol（例如 "TICKET"）
                uri: URI.to_string(),       // 指向 JSON 元数据的 URL（存储图像、描述等）
                seller_fee_basis_points: 0, // 没有转售版税
                creators: None,
                collection: None,           // 稍后再设置 collection
                uses: None,
            },
            true,             // 是否可修改
            true, // 是否是可销售的 primary sale
            None,
        )?;
        // 创建 master edition（每张票是唯一 NFT，所以 edition 设为 0）
        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    payer: ctx.accounts.payer.to_account_info(),
                    mint: ctx.accounts.ticket_mint.to_account_info(),
                    edition: ctx.accounts.master_edition.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            Some(0), // edition number
        )?;
        // 设置 NFT 归属某集合（用于后续统一管理，例如抽奖集合）
        // verify nft as part of collection
        set_and_verify_sized_collection_item(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                SetAndVerifySizedCollectionItem {
                    metadata: ctx.accounts.metadata.to_account_info(),
                    collection_authority: ctx.accounts.collection_mint.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    collection_mint: ctx.accounts.collection_mint.to_account_info(),
                    collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
                    collection_master_edition: ctx
                        .accounts
                        .collection_master_edition
                        .to_account_info(),
                },
                &signer_seeds,
            ),
            None,
        )?;
        // 当前抽奖票数 +1（供下一票编号使用）
        ctx.accounts.token_lottery.ticket_num += 1;

        Ok(())
    }

    // 提交随机数结果，记录 randomness 来源
    /**
        该函数是整个抽奖流程中的 “提交随机数阶段”，由管理员调用：
        确保该阶段只会提交一个新的随机数（slot 校验）；
        使用 Switchboard 提供的 VRF 随机数；
        随机数数据将被存储在 token_lottery 中的 randomness_account 字段，为接下来选择中奖者（choose_winner）做准备。
        我有一个随机数种子，先存档
        随机种子（Random Seed）在抽奖里的作用
            1、随机种子 = 一个不可预测的随机值（来自 Switchboard VRF / Chainlink VRF 等）。
            2、每次抽奖 生成新的随机种子，用它去确定赢家。
        这样做的好处：
        1、避免重复开奖
            如果不换随机数，每次调用 choose_a_winner 都可能得到同样的结果。
            每次生成新的随机数，保证每次开奖都是独立随机事件。
        2、防止操纵 / 提前知道结果
            commit-reveal 模式：先提交随机数账户（commit_a_winner），再根据随机数选中奖者（choose_a_winner）。
            在随机数提交前，没人能预测谁会中奖。
        3、保证公平性
            随机数和票号（用户）通过取模映射，保证每张票有相同概率中奖。
        4、流程总结
            1、用户买票 → 分配票号（ticket_num 自增）
            2、管理员 / 自动化触发随机数生成 → commit_a_winner 提交随机种子
            3、随机数生成完成 → choose_a_winner 根据随机数计算中奖票号
            4、中奖用户调用 claim_prize 领取奖励
    **/
    pub fn commit_a_winner(ctx: Context<CommitWinner>) -> Result<()> {
        // 获取当前区块链的时间（包含 slot、timestamp 等信息）
        let clock = Clock::get()?;

        // 获取 token_lottery 状态账户的可变引用
        let token_lottery = &mut ctx.accounts.token_lottery;
        // 校验调用者是否为合约管理员（即 token_lottery.authority）
        if ctx.accounts.payer.key() != token_lottery.authority {
            return Err(ErrorCode::NotAuthorized.into());
        }
        // 从 randomness_account_data 中解析出 Switchboard 提供的随机数结果
        let randomness_data = RandomnessAccountData::parse(ctx.accounts.randomness_account_data.data.borrow()).unwrap();
        // 校验该随机数的 seed_slot 是否为上一个 slot，确保是“最新”的未被提交过的随机数
        if randomness_data.seed_slot != clock.slot - 1 {
            return Err(ErrorCode::RandomnessAlreadyRevealed.into());
        }
        // 将本次随机数账户地址存储在 token_lottery 状态中，为之后选出赢家做准备
        token_lottery.randomness_account = ctx.accounts.randomness_account_data.key();

        Ok(())
    }

    // 基于 randomness 选择中奖票号
    /**
        这是 抽奖系统的开奖函数，由管理员在抽奖结束后调用。它主要实现了以下功能：
        校验调用者身份和传入账户是否正确；
        校验抽奖时间是否已结束；
        使用已提交的 VRF 随机数，确定中奖票号；
        将中奖票号写入状态并锁定不可再次开奖。
    **/
    pub fn choose_a_winner(ctx: Context<ChooseWinner>) -> Result<()> {
        // 获取当前 slot 和区块时间等链上时间信息
        let clock = Clock::get()?;
        // 获取 token_lottery 状态账户的可变引用
        let token_lottery = &mut ctx.accounts.token_lottery;

        // 校验传入的随机数账户是否与之前 commit 的一致
        if ctx.accounts.randomness_account_data.key() != token_lottery.randomness_account {
            return Err(ErrorCode::IncorrectRandomnessAccount.into());
        }
        // 校验调用者是否为管理员，即 authority
        if ctx.accounts.payer.key() != token_lottery.authority {
            return Err(ErrorCode::NotAuthorized.into());
        }
        // 检查当前是否已经到达抽奖结束 slot，确保在开奖时间之后执行
        if clock.slot < token_lottery.lottery_end {
            msg!("Current slot: {}", clock.slot);
            msg!("End slot: {}", token_lottery.lottery_end);
            return Err(ErrorCode::LotteryNotCompleted.into());
        }
        // 检查是否已经选择过赢家，防止重复选择
        require!(token_lottery.winner_chosen == false, ErrorCode::WinnerChosen);
        // 从 Switchboard 随机数账户中提取随机值（已解密）
        let randomness_data = 
            RandomnessAccountData::parse(ctx.accounts.randomness_account_data.data.borrow()).unwrap();
        // 使用当前 slot 获取真实的随机值（reveal 阶段）
        let revealed_random_value = randomness_data.get_value(&clock)
            .map_err(|_| ErrorCode::RandomnessNotResolved)?;
        // 打印随机值和票数，方便调试
        msg!("Randomness result: {}", revealed_random_value[0]);
        msg!("Ticket num: {}", token_lottery.ticket_num);
        // 取随机值对票数取模，得到赢家的票号（范围：0 ~ ticket_num-1）
        let randomness_result = 
            revealed_random_value[0] as u64 % token_lottery.ticket_num;
        // 打印最终赢家的票号
        msg!("Winner: {}", randomness_result);
        // 将赢家票号记录到 token_lottery 中，并标记为已开奖
        token_lottery.winner = randomness_result;
        token_lottery.winner_chosen = true;

        Ok(())
    }
    // 	中将者领取奖池 SOL
    /**
        这个函数的主要职责是 验证中奖者身份 和 将奖池 SOL 奖金发送到中奖钱包，通过多个检查保证：
        抽奖已完成；
        用户提交了正确的 NFT；
        NFT 属于本次抽奖的集合（Collection）；
        用户确实持有该 NFT；
        避免重复领奖。
    **/
    pub fn claim_prize(ctx: Context<ClaimPrize>) -> Result<()> {
        // Check if winner has been chosen
        // Step 1: 检查是否已经选择了中奖者
        msg!("Winner chosen: {}", ctx.accounts.token_lottery.winner_chosen);
        require!(ctx.accounts.token_lottery.winner_chosen, ErrorCode::WinnerNotChosen);

        // Check if token is a part of the collection
        // Step 2: 检查 NFT 是否属于指定的 Collection 且已验证
        // - `collection.verified`: 表示该 NFT 的 Collection 已通过验证（一般由 Collection 创建者签名）
        // - `collection.key == collection_mint.key()`: 检查 NFT 属于本次抽奖使用的 Collection
        require!(ctx.accounts.metadata.collection.as_ref().unwrap().verified, ErrorCode::NotVerifiedTicket);
        require!(ctx.accounts.metadata.collection.as_ref().unwrap().key == ctx.accounts.collection_mint.key(), ErrorCode::IncorrectTicket);
        // Step 3: 构造中奖票据名称，格式为 "Ticket" + 中奖号码，如 "Ticket42"
        let ticket_name = NAME.to_owned() + &ctx.accounts.token_lottery.winner.to_string();
        // Step 4: 获取实际 NFT Metadata 中记录的名称（去除空字符）
        let metadata_name = ctx.accounts.metadata.name.replace("\u{0}", "");


        msg!("Ticket name: {}", ticket_name);
        msg!("Metdata name: {}", metadata_name);

        // Check if the winner has the winning ticket
        // Step 5: 验证用户提交的 NFT 是否为中奖票据（名称匹配）
        require!(metadata_name == ticket_name, ErrorCode::IncorrectTicket);
        // Step 6: 确保该 NFT 的持有账户（ATA）中余额大于 0，说明用户确实拥有该 NFT
        require!(ctx.accounts.destination.amount > 0, ErrorCode::IncorrectTicket);
        // Step 7: 将奖池资金从合约账户（PDA）转账到中奖用户的钱包（payer）
        **ctx.accounts.token_lottery.to_account_info().try_borrow_mut_lamports()? -= ctx.accounts.token_lottery.lottery_pot_amount;
        **ctx.accounts.payer.try_borrow_mut_lamports()? += ctx.accounts.token_lottery.lottery_pot_amount;
        // Step 8: 清空奖池金额，避免重复领奖
        ctx.accounts.token_lottery.lottery_pot_amount = 0;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    // 调用者（领奖者），需要是中奖票 NFT 的持有者，签名者
    #[account(mut)]
    pub payer: Signer<'info>,
    // token_lottery 抽奖状态账户，用于读取中奖票号、奖池金额等
    // 使用固定种子 `"token_lottery"` 初始化，必须与购票和开奖使用的是同一个
    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,
    // Collection NFT 的 mint（表示整个票据集合的根 mint）
    // 用于验证中奖票是否属于该集合
    #[account(
        mut,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,
    // 中奖票 NFT 的 mint（即中奖的具体 NFT）
    // 种子是：中奖票号（token_lottery.winner）对应的 ticket mint PDA
    #[account(
        seeds = [token_lottery.winner.to_le_bytes().as_ref()],
        bump,
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,
    // 中奖票 NFT 的元数据账户（用于读取 name 与 collection 信息进行校验）
    #[account(
        seeds = [b"metadata", token_metadata_program.key().as_ref(), ticket_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata: Account<'info, MetadataAccount>,
    // 调用者（payer）的钱包中，与中奖票 mint 对应的 token account
    // 要求该账户必须存在，并且持有 NFT（amount > 0）
    #[account(
        associated_token::mint = ticket_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,
    // collection mint 的元数据账户
    // 用于验证票 NFT 是否属于这个集合（collection.verified = true 且 key 匹配）
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), collection_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub collection_metadata: Account<'info, MetadataAccount>,

    // SPL Token 程序接口（用在 token 相关操作上）
    pub token_program: Interface<'info, TokenInterface>,
    // 系统程序（用于 lamports 转账）
    pub system_program: Program<'info, System>,
    // Metaplex 的 Token Metadata 程序（用于验证元数据和集合归属）
    pub token_metadata_program: Program<'info, Metadata>,
}

#[derive(Accounts)]
pub struct CommitWinner<'info> {
    // 调用者（必须是管理员），提交随机数结果的 signer
    #[account(mut)]
    pub payer: Signer<'info>,
    // token_lottery 抽奖状态账户
    // 存储抽奖基本信息，包括 authority、randomness_account 等
    // 使用固定种子 `"token_lottery"` 创建
    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// CHECK: The account's data is validated manually within the handler.
    // Switchboard V2 生成的随机数账户
    // CHECK: 数据结构不由 Anchor 自动校验，需在逻辑中手动解析校验
    pub randomness_account_data: UncheckedAccount<'info>,
    // 系统程序（用于执行系统调用或 lamports 检查）
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ChooseWinner<'info> {
    // 调用者，必须是管理员（`token_lottery.authority`）
    // 只有管理员有权限执行开奖操作
    #[account(mut)]
    pub payer: Signer<'info>,
    // 抽奖状态账户，包含票数、开奖时间、是否已开奖、中奖号码等状态
    // 使用固定种子 `"token_lottery"` 生成
    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// CHECK: The account's data is validated manually within the handler.
    // 提交过的 Switchboard 随机数账户（必须与 commit 阶段记录的一致）
    pub randomness_account_data: UncheckedAccount<'info>,
    // 系统程序（用于执行系统调用或 lamports 检查）
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyTicket<'info> {
    // 用户的钱包地址，作为购票者，需要签名并支付票价（SOL）
    #[account(mut)]
    pub payer: Signer<'info>,
    // 抽奖状态账户，记录票价、已售票数量等信息
    // 用固定种子 `"token_lottery"` 创建，需与初始化时保持一致
    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump
    )]
    pub token_lottery: Account<'info, TokenLottery>,
    // 要 mint 出来的票据 NFT 的 mint 账户（本张票）
    // 通过已售票数 `ticket_num` 作为种子创建
    #[account(
        init,
        payer = payer,
        seeds = [token_lottery.ticket_num.to_le_bytes().as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
        mint::token_program = token_program
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,
    // 用户接收 NFT 的 Token Account，绑定票 NFT 和用户钱包
    // 自动与 `ticket_mint` 和 `payer` 生成绑定
    #[account(
        init,
        payer = payer,
        associated_token::mint = ticket_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,
    // 票 NFT 的 Metadata 账户（由 Metaplex 负责创建和填充）
    // CHECK: 不由 Anchor 校验，因此必须手动初始化并验证
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), 
        ticket_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account will be initialized by the metaplex program
    pub metadata: UncheckedAccount<'info>,
    // Master Edition 账户（由 Metaplex 创建，标识此 NFT 为主版本）
    // CHECK: 不由 Anchor 校验，因此必须手动初始化并验证
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), 
            ticket_mint.key().as_ref(), b"edition"],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account will be initialized by the metaplex program
    pub master_edition: UncheckedAccount<'info>,
    // Collection NFT 的 metadata 账户（用于后续 collection 验证）
    // CHECK: 不由 Anchor 校验
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), collection_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account will be initialized by the metaplex program
    pub collection_metadata: UncheckedAccount<'info>,
    // Collection NFT 的 Master Edition 账户
    // CHECK: 不由 Anchor 校验
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), 
            collection_mint.key().as_ref(), b"edition"],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account will be initialized by the metaplex program
    pub collection_master_edition: UncheckedAccount<'info>,
    // Collection 的 mint 账户（用于作为子 NFT 的 mint authority）
    #[account(
        mut,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    // Anchor 所需的 Associated Token Program（用于初始化 ATA）
    pub associated_token_program: Program<'info, AssociatedToken>,
    // SPL Token 接口（用于 mint、transfer 操作）
    pub token_program: Interface<'info, TokenInterface>,
    // 系统程序（支付 SOL、创建账户）
    pub system_program: Program<'info, System>,
    // Metaplex 的 Token Metadata 程序，用于创建 metadata、edition 和 collection 设置
    pub token_metadata_program: Program<'info, Metadata>,
    // 租金账户，供系统初始化新账户时扣除租金参考
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeConifg<'info> {
    // 调用者，用于支付初始化账户的租金和费用，同时被记录为抽奖管理员
    #[account(mut)]
    pub payer: Signer<'info>,
    // 抽奖配置账户，用于存储整个抽奖活动的核心状态数据（初始化创建）
    // 使用固定种子 `"token_lottery"` + bump 生成 PDA
    // `init` 表示第一次创建，`space` 指定数据空间大小（8 + struct 大小）
    #[account(
        init,
        payer = payer,
        space = 8 + TokenLottery::INIT_SPACE,
        // Challenge: Make this be able to run more than 1 lottery at a time
        seeds = [b"token_lottery".as_ref()],
        bump
    )]
    pub token_lottery: Box<Account<'info, TokenLottery>>,
    // solana的系统程序（用于创建账户、转账 lamports）
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeLottery<'info> {
    // 调用者，通常是管理员，支付初始化 Collection NFT 所需费用
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Collection NFT 的 Mint 账户（代表所有票 NFT 的集合）
    /// 由程序使用 `"collection_mint"` 固定种子 + bump 创建
    /// mint authority 和 freeze authority 均设置为自身 PDA
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Collection NFT 对应的 Metadata 账户（Metaplex 初始化）
    /// 用于存储 name、symbol、uri、collection 属性
    /// CHECK: Metaplex CPI 内部会初始化和填充此账户
    /// CHECK: This account will be initialized by the metaplex program
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// Collection NFT 的 Master Edition 账户（标记其为唯一主版本）
    /// CHECK: 同样由 Metaplex CPI 初始化
    /// CHECK: This account will be initialized by the metaplex program
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// Collection NFT 的 Token Account（表示该 NFT 当前存在哪个账户中）
    /// 使用固定种子 `"collection_token_account"` 创建，用于接收 mint 的 NFT
    /// authority 设置为自己（和 mint authority 匹配）
    #[account(
        init_if_needed,
        payer = payer,
        seeds = [b"collection_token_account".as_ref()],
        bump,
        token::mint = collection_mint,
        token::authority = collection_token_account
    )]
    pub collection_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// SPL Token 接口（用于 mint 操作）
    pub token_program: Interface<'info, TokenInterface>,
    /// Anchor 的 Associated Token Program，用于自动初始化 token_account（ATA）
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// 系统程序，用于创建账户/转账/分配租金等
    pub system_program: Program<'info, System>,
    /// Metaplex Metadata 程序，用于创建 Metadata 和 Master Edition
    pub token_metadata_program: Program<'info, Metadata>,
    /// 租金系统变量，用于创建账户时参考当前最小余额
    pub rent: Sysvar<'info, Rent>,
}

#[account]
#[derive(InitSpace)]
pub struct TokenLottery {
    // PDA bump，用于验证 `token_lottery` 账户 PDA（与 seeds 一起生成 PDA）
    pub bump: u8,
    // 抽奖的最终中奖号码（ticket 编号，对应 NFT 的种子
    pub winner: u64,
    // 是否已经选择过中奖者，防止重复开奖
    pub winner_chosen: bool,
    // 抽奖开始的 slot（即什么时候可以开始购票）
    pub lottery_start: u64,
    // 抽奖结束的 slot（即什么时候截止购票 & 开始开奖）
    pub lottery_end: u64,
    // Is it good practice to store SOL on an account used for something else?
    // 奖池累计的 SOL 总额（每张票价都会累加进来）
    pub lottery_pot_amount: u64,
    // 当前已售出票数量，每卖出一张票就会自增（作为 ticket mint 的种子）
    pub ticket_num: u64,
    // 每张票的价格（单位为 lamports）
    pub price: u64,
    // 抽奖使用的 Switchboard randomness 账户地址（commit 阶段写入）
    pub randomness_account: Pubkey,
    // 抽奖发起者 / 管理员（只有该地址可以开奖、提交 randomness）
    pub authority: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    /// 用于验证 randomness_account 是否与 commit 阶段记录一致
    #[msg("Incorrect randomness account")]
    IncorrectRandomnessAccount,

    /// 当前尚未达到开奖时间，禁止提前开奖
    #[msg("Lottery not completed")]
    LotteryNotCompleted,

    /// 当前时间不在允许购票的时间段内（start ~ end）
    #[msg("Lottery is not open")]
    LotteryNotOpen,

    /// 当前 signer 不是管理员（token_lottery.authority）
    #[msg("Not authorized")]
    NotAuthorized,

    /// 提交的 randomness_account 已经被揭示过，不能重复使用
    #[msg("Randomness already revealed")]
    RandomnessAlreadyRevealed,

    /// randomness_account 尚未准备好（未产生随机数或无效）
    #[msg("Randomness not resolved")]
    RandomnessNotResolved,

    /// 抽奖尚未开奖，禁止领取奖金
    #[msg("Winner not chosen")]
    WinnerNotChosen,

    /// 抽奖已经开奖，不能再次开奖
    #[msg("Winner already chosen")]
    WinnerChosen,

    /// NFT 的 Metadata 中未标记为已加入 collection
    #[msg("Ticket is not verified")]
    NotVerifiedTicket,

    /// 当前 NFT 不是中奖票（ticket 名称或 collection 验证失败）
    #[msg("Incorrect ticket")]
    IncorrectTicket,
}


/***
    在 Solana 上没有 Chainlink Automation 这种「官方统一服务」，但有几种替代方案：

    Clockwork → 原生的自动调度协议（最接近 Chainlink Automation）。

    Switchboard Crank → 结合 RNG 的自动化触发，和你现有代码完美配合。

    自建 bot → 监听链上事件，到点发交易（最常见，灵活）。

    正是这种工具可以帮你触发刚才提到的 commit_a_winner、choose_a_winner、claim_prize 这些函数。

    在 Solana 生态里，智能合约（Program）不会自动执行，只能在有外部交易时被调用。所以要实现“自动化执行”，
    就需要借助类似 定时执行器 / 自动化服务 来替代 Chainlink Automation 的角色。

    常见方式有：

    1、Switchboard Functions（推荐）
        类似 Chainlink Automation 的 “keeper” 机制。
        你可以定义一个条件（比如到达开奖时间），让 Switchboard 在链下监控，当条件满足时，它会发起一个交易，调用你的 commit_a_winner 或 choose_a_winner。
        好处是和 Solana 原生集成，已经有人用它做定时开奖、清算等逻辑。

    2、Cronos / Clockwork（去中心化调度）
        允许你在链上定义一个 “Cron job”，比如“每天 20:00 调用我的 choose_a_winner”。
        Clockwork 会确保有人帮你发交易触发。

    3、自己搭建 Off-chain Worker（中心化方式）
        写个脚本（Rust / Go / JS），跑在服务器上。
        脚本定时检查链上状态（比如开奖时间），然后用管理员的钱包发起交易，调用 choose_a_winner。
        缺点是去中心化不足，但最容易实现。

    🔑 总结：
    你提到的这些函数 不会自动执行，必须有人发交易。
    → 在 Solana 里，可以用 Switchboard Functions 或 Clockwork 这样的自动化工具来代替 Chainlink Automation。
    → 如果不想依赖外部服务，也可以写个 后台脚本 来定时触发。
    要不要我给你写一个 基于 Switchboard Functions 定时触发 choose_a_winner 的示例？
 */