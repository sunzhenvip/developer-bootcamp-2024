

主要功能确认：
    1、购票流程：
        自动生成 NFT（每个 ticket 是一个独立的 NFT mint）。
        使用 mint_to + create_metadata_accounts_v3 + create_master_edition_v3 来生成标准 NFT。
        使用 set_and_verify_sized_collection_item 把票加入集合，确保后续可验证。
    2、抽奖流程：
        使用 Switchboard 的随机数来源，确保公平性。
        中奖号码存储在 token_lottery.winner 中。
    3、领奖流程：
        验证用户是否拥有中将 NFT（基于元数据 name 和 collection 校验）。
        转移账户 lamports 给中奖人。
注意点和优化建议：
    硬编码 PDA 种子问题（多次抽奖会冲突）
        seeds = [b"token_lottery".as_ref()]
    目前只允许运行一个抽奖，未来如果支持多个，可以用 lottery_id 或 Pubkey 作为种子的一部分：
        seeds = [b"token_lottery", lottery_id.as_ref()]

    Token 转账逻辑：在领奖环节，使用：
        **ctx.accounts.token_lottery.to_account_info().try_borrow_mut_lamports()? -= ctx.accounts.token_lottery.lottery_pot_amount;
        **ctx.accounts.payer.try_borrow_mut_lamports()? += ctx.accounts.token_lottery.lottery_pot_amount;

        这种 lamports 转账方式可行，但：不能转 SPL token，只能转 SOL

        建议在账户初始化时显式设为系统账户 若未来需要多种币种奖励，考虑引入 Token SPL reward 方式