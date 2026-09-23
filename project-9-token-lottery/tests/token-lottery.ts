import * as anchor from "@coral-xyz/anchor";
// Switchboard On-Demand SDK：创建 Randomness Account，并构造 commit/reveal 指令。
import * as sb from "@switchboard-xyz/on-demand";
import { Program } from "@coral-xyz/anchor";
// anchor build 后根据 Lottery IDL 生成的 TS 类型，为 methods/accounts 提供类型提示。
import { TokenLottery } from "../target/types/token_lottery";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
// 根据「Mint + 钱包 owner」确定性计算 Associated Token Account 地址。
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
// 当前测试没有直接使用 NodeWallet；它是早期独立 Switchboard 调试代码的遗留导入。
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { Idl } from "@coral-xyz/anchor/dist/cjs/idl";
describe("token-lottery", () => {
  // AnchorProvider.env() 从 Anchor 环境中取得 RPC、测试钱包和 commitment。
  // 本项目的 RPC 指向手动启动的 http://127.0.0.1:8899。
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;
  anchor.setProvider(provider);

  // anchor.workspace 根据 target/idl/token_lottery.json 创建 Lottery 客户端。
  // 这是“构造指令的 TS 对象”，不是链上的 Rust Program 本体。
  const program = anchor.workspace.TokenLottery as Program<TokenLottery>;
  // 在 before() 中根据 Switchboard IDL 创建，并连接同一个本地 provider。
  let switchboardProgram: Program<any>;
  let metaDataProgramLength: any;
  // Randomness Account 不是 PDA，而是新 Keypair 对应的普通账户。
  // 创建该账户时 rngKp 必须签名，commit/reveal 随后都使用同一个地址。
  const rngKp = anchor.web3.Keypair.generate();

  const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
  );
  const apiKey = "c5730fdb-3471-42ff-92ad-97256fa83871";

  // 可选辅助函数：从主网下载最新 Switchboard IDL 并保存到 tests 目录。
  // 正常测试没有调用它，而是直接读取已经保存的 switchboard-idl.json。
  async function getSwitchboardIDL() {
    const switchboardIDL: Idl | null = await anchor.Program.fetchIdl(
      sb.ON_DEMAND_MAINNET_PID, // sb.SB_ON_DEMAND_PID,一开始是这个应该是写错了
      {
        connection: new anchor.web3.Connection(
          "https://mainnet.helius-rpc.com/?api-key=" + apiKey
        ),
      }
    );
    // 主网上没有找到 IDL 时 fetchIdl 会返回 null，必须先检查再创建 Program。
    if (!switchboardIDL) {
      throw new Error("Failed to fetch IDL: returned null");
    }
    switchboardProgram = new anchor.Program(switchboardIDL, provider);
    let fs = require("fs");
    fs.writeFile(
      "tests/switchboard-idl.json",
      JSON.stringify(switchboardIDL),
      function (err: Error | null) {
        if (err) throw err;
        console.log("The file has been saved!");
      }
    );
  }
  // 没有 api-key 可以在这个网站注册获取一个 免费的 有速率限制 每秒钟几个 https://www.helius.dev/
  before("Loading switchboard program", async () => {
    /*await provider.connection.requestAirdrop(
      rngKp.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 100  // 空投 2 SOL
    );*/
    // IDL 描述 Switchboard 的指令和账户布局；provider 仍指向本地 validator，
    // 所以后续 Queue/Oracle/Randomness Account 都从本地链读取。
    const switchboardIDL = require("../tests/switchboard-idl.json"); // 本地 IDL 文件
    switchboardProgram = new anchor.Program(switchboardIDL, provider);

    // 确认 start-validator.sh 已把 Metaplex Metadata Program 加载到本地链。
    // 如果 accountInfo 为 null，后面的 NFT Metadata CPI 一定会失败。
    const accountInfo = await connection.getAccountInfo(
      TOKEN_METADATA_PROGRAM_ID
    );
    metaDataProgramLength = accountInfo?.data.length;
  });

  // 环境冒烟测试：只确认外部 Program 和随机数账户公钥准备正常，不修改业务状态。
  it("测试是否正常获取数据", async () => {
    console.log(
      "ondemand.so 合约公钥地址",
      switchboardProgram.programId.toString()
    );
    console.log("metadata.so 账户存储字节", metaDataProgramLength);
    console.log("rngKp.publicKey", rngKp.publicKey.toString());
  });
  // console.log("已退出");
  // return

  // 购买一张彩票的客户端封装。Rust 的 buy_ticket 会在一条指令中完成：
  // 支付票款 -> 创建 Ticket Mint/ATA -> mint NFT -> 创建 Metadata/Edition
  // -> 验证其属于 Lottery Collection -> ticket_num + 1。
  async function buyTicket() {
    // .instruction() 只构造指令，不发送交易。除了 tokenProgram 外，其余账户
    // 大多由 Anchor 根据 IDL 中的 seeds、payer、associated_token 约束自动推导。
    const buyTicketIx = await program.methods
      .buyTicket()
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    // 每笔 Solana 交易必须携带近期 blockhash，并在 lastValidBlockHeight 前执行。
    const blockhashContext = await connection.getLatestBlockhash();

    // buy_ticket 内有多次 Token/Metaplex CPI，显式提高计算上限避免 CU 不足。
    const computeIx = anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 300000,
    });

    // 每个 Compute Unit 支付 1 micro-lamport 优先费；本地测试主要用于演示。
    const priorityIx = anchor.web3.ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: 1,
    });

    const tx = new anchor.web3.Transaction({
      blockhash: blockhashContext.blockhash,
      lastValidBlockHeight: blockhashContext.lastValidBlockHeight,
      feePayer: wallet.payer.publicKey,
    })
      .add(buyTicketIx)
      .add(computeIx)
      .add(priorityIx);

    // 测试钱包同时是 payer 和购票者，支付手续费、账户租金与彩票价格。
    const sig = await anchor.web3.sendAndConfirmTransaction(connection, tx, [
      wallet.payer,
    ]);
    console.log("buy ticket ", sig);
  }

  it("Is initialized!", async () => {
    // 项目使用 slot 而不是 Unix 时间。结束位置设为当前 slot + 10，
    // 完成初始化、买票和随机数交易后通常已经到达开奖 slot。
    const slot = await connection.getSlot();
    console.log("Current slot", slot);

    // Lottery Collection Mint PDA，必须与 Rust seeds=[b"collection_mint"] 一致。
    const mint = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("collection_mint")],
      program.programId
    )[0];

    // Metadata 和 Master Edition 由 Metaplex Program 的 seeds 推导，所以这里
    // 使用 TOKEN_METADATA_PROGRAM_ID，而不是 Lottery Program ID。
    const metadata = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    )[0];

    const masterEdition = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
        Buffer.from("edition"),
      ],
      TOKEN_METADATA_PROGRAM_ID
    )[0];

    // 创建 token_lottery 状态 PDA：立即开放、10 个 slot 后结束、票价 10,000 lamports。
    const initConfigIx = await program.methods
      .initializeConfig(
        new anchor.BN(0),
        new anchor.BN(slot + 10),
        new anchor.BN(10000)
      )
      .instruction();

    // 创建 Collection Mint，并通过 CPI 创建 Collection Metadata/Master Edition。
    const initLotteryIx = await program.methods
      .initializeLottery()
      .accounts({
        masterEdition: masterEdition,
        metadata: metadata,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    const blockhashContext = await connection.getLatestBlockhash();

    // 两条初始化指令在同一交易中原子执行：任何一条失败，全部修改都会回滚。
    const tx = new anchor.web3.Transaction({
      blockhash: blockhashContext.blockhash,
      lastValidBlockHeight: blockhashContext.lastValidBlockHeight,
      feePayer: wallet.payer.publicKey,
    })
      .add(initConfigIx)
      .add(initLotteryIx);

    const sig = await anchor.web3.sendAndConfirmTransaction(connection, tx, [
      wallet.payer,
    ]);
    console.log(sig);
  });

  it("Is buying tickets!", async () => {
    // 同一 wallet 连续购买 5 张票，所以中奖号码无论是 0~4 中哪一个，
    // 测试钱包都持有相应 NFT，后面的 claim 测试一定能找到中奖票。
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();
  });

  it("Is committing and revealing a winner", async () => {
    /*
     * 开奖阶段一共发送三笔交易：
     *
     * 交易 1：创建 Switchboard Randomness Account。
     * 交易 2：[Switchboard commit, Lottery commit_a_winner]。
     * 交易 3：[Switchboard reveal, Lottery choose_a_winner]。
     *
     * 交易 2、3 都把两条指令放在同一笔交易里，并按照数组顺序执行。
     * 后一条指令可以立即读取前一条刚写入的账户状态；如果任一指令失败，
     * 同一笔交易中的全部状态修改都会回滚。
     */

    // 课程使用的 Switchboard On-Demand 主网 Queue 地址。
    // setup-local.sh 会下载该 Queue 和 Oracle 的主网账户快照，本测试的交易
    // 仍然只发送到本地 validator，不会在主网上创建 Randomness Account。
    // queue_addr 只保留作地址文字说明；下面真正参与 SDK 调用的是 PublicKey queue。
    const queue_addr = "A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w"; // switchboard
    const queue = new anchor.web3.PublicKey(
      "A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w"
    );

    // Queue 是可以为随机数请求提供服务的 Oracle 候选池。
    const queueAccount = new sb.Queue(switchboardProgram, queue);
    console.log("Queue account", queue.toString());
    try {
      const loadData = await queueAccount.loadData();
      // oracleKeys 是固定容量数组，length 可能为 78，不代表实际有 78 个成员；
      // 当前有效成员数应看 oracleKeysLen。setup 脚本也是按 oracleKeysLen 截取。
      console.log("await queueAccount.loadData() ", loadData.oracleKeys.length);
      /*for (let i = 0; i < loadData.oracleKeys.length; i++) {
        console.log(loadData.oracleKeys[i].toString())
      }*/
    } catch (err) {
      console.error("❌ Queue account not found:", err);
      process.exit(1);
    }

    // 构造 Randomness Account 初始化指令：
    // - randomness：后续 loadData/commitIx/revealIx 使用的 TS 包装对象；
    // - ix：真正交给 Switchboard Program 执行的链上初始化指令；
    // - rngKp.publicKey：新 Randomness Account 地址，并与上面的 Queue 绑定。
    const [randomness, ix] = await sb.Randomness.create(
      switchboardProgram,
      rngKp,
      queue
    );
    console.log("Created randomness account..");
    console.log("Randomness account", randomness.pubkey.toBase58());
    console.log("rkp account", rngKp.publicKey.toBase58());
    // Switchboard 指令使用的账户较多，SDK 用 v0 Versioned Transaction 构造交易，
    // 并配置计算预算。rngKp 必须签名，因为交易正在创建它对应的账户。
    const createRandomnessTx = await sb.asV0Tx({
      connection: connection,
      ixs: [ix],
      payer: wallet.publicKey,
      signers: [wallet.payer, rngKp],
      computeUnitPrice: 75_000,
      computeUnitLimitMultiple: 1.3,
    });

    // 保存 blockhash 和 lastValidBlockHeight，确认交易时用于限定有效期。
    // 更严格的生产客户端通常会在每次发送交易前重新获取最新 blockhash。
    const blockhashContext = await connection.getLatestBlockhashAndContext();

    const createRandomnessSignature = await connection.sendTransaction(
      createRandomnessTx
    );
    await connection.confirmTransaction({
      signature: createRandomnessSignature,
      blockhash: blockhashContext.value.blockhash,
      lastValidBlockHeight: blockhashContext.value.lastValidBlockHeight,
    });
    console.log(
      "Transaction Signature for randomness account creation: ",
      createRandomnessSignature
    );
    // 这里加载的是刚创建的 Randomness Account，并不是 Queue Account；
    // queueData 是原示例留下的变量名。
    const queueData = await randomness.loadData();
    /*报错详情 TypeError: Cannot read properties of null (reading 'account')
    at /data/network/rust/web3/sunzhenvip/developer-bootcamp-2024/project-9-token-lottery/node_modules/@switchboard-xyz/on-demand/src/accounts/queue.ts:791:60
    at Array.map (<anonymous>)
    at Queue.<anonymous> (node_modules/@switchboard-xyz/on-demand/src/accounts/queue.ts:791:8)
    at Generator.next (<anonymous>)
    at fulfilled (node_modules/@switchboard-xyz/on-demand/dist/cjs/accounts/queue.js:38:58)
    at processTicksAndRejections (node:internal/process/task_queues:95:5)
    error Command failed with exit code 1.
    info Visit https://yarnpkg.com/en/docs/cli/run for documentation about this command.
    */
    console.log("Queue data", queueData.authority.toString());

    /*
     * randomness.commitIx(queue) 的客户端工作：
     *
     * 1. 从 Queue 读取当前 Oracle 公钥列表；
     * 2. 批量读取对应的 Oracle Account；
     * 3. 测试各 Oracle 的 gateway 是否可以访问；
     * 4. 筛选 verificationStatus=4、validUntil 至少还有 1 小时的 Oracle；
     * 5. 从有效 Oracle 中选择一个；
     * 6. 返回 Switchboard randomnessCommit 链上指令。
     *
     * commit 阶段还没有取得最终随机结果。它会在链上锁定本次使用的 Oracle、
     * seed slot 和 slot hash，避免调用者看到结果以后再更换随机数来源。
     *
     * 之前的 null.account 错误发生在第 2 步：主网 Queue 已经轮换到新成员，
     * 旧 start-validator 脚本却只加载 2024 年写死的旧 Oracle。本地查询当前
     * Oracle 地址时得到 null，旧版 SDK 随后直接读取 null.account 而报错。
     */
    const sbCommitIx = await randomness.commitIx(queue); // 原报错点（现已修复）
    console.log("sbCommitIx", sbCommitIx.programId.toString());

    // 这是 Lottery Program 自己的 commit_a_winner 指令，它不负责产生随机数。
    // Rust 端会验证：
    // 1. payer 是彩票 authority；
    // 2. Randomness Account 的 seed_slot 足够新；
    // 3. 把 randomness.pubkey 保存到 token_lottery.randomness_account，
    //    从而锁定 reveal/开奖必须使用同一个 Randomness Account。
    const commitIx = await program.methods
      .commitAWinner()
      .accounts({
        randomnessAccountData: randomness.pubkey,
      })
      .instruction();

    // 指令顺序不能交换：Switchboard 先 commit 并更新 Randomness Account，
    // Lottery 的 commitAWinner 随后读取新状态。两条指令在同一交易中原子执行。
    const commitTx = await sb.asV0Tx({
      connection: switchboardProgram.provider.connection,
      ixs: [sbCommitIx, commitIx],
      payer: wallet.publicKey,
      signers: [wallet.payer],
      computeUnitPrice: 75_000,
      computeUnitLimitMultiple: 1.3,
    });

    const commitSignature = await connection.sendTransaction(commitTx);
    await connection.confirmTransaction({
      signature: commitSignature,
      blockhash: blockhashContext.value.blockhash,
      lastValidBlockHeight: blockhashContext.value.lastValidBlockHeight,
    });
    console.log("✅ Transaction Signature for commit: ", commitSignature);
    // 原来的失败点在前面的 randomness.commitIx(queue)：旧脚本加载的是已经
    // 退出 Queue 的 2024 年 Oracle，所以 SDK 查询账户后得到 null.account。
    // setup-local.sh 现在会同步当前成员并准备一个可用的本地 Oracle，因此
    // commit 成功后这里才能从同一个 randomness account 生成 reveal 指令。

    /*
     * revealIx 与 commitIx 的关键区别：
     *
     * - commitIx 选择有效 Oracle，并构造链上的 commit 指令；
     * - revealIx 读取 commit 时选中的 Oracle，从 Oracle Account 取 gateway_uri；
     * - SDK 通过 HTTP 把 seed slot/slot hash 发给真实 Oracle Gateway；
     * - Gateway 在 TEE 中计算随机值并返回签名；
     * - SDK 把随机值、签名、recovery id 封装为 randomnessReveal 链上指令。
     *
     * 因此测试交易虽然在本地 validator 执行，调用 revealIx 时仍需要互联网。
     * Oracle 的真实私钥和 TEE 无法从主网账户 JSON 快照复制到本地。
     */
    const sbRevealIx = await randomness.revealIx();

    // Lottery 的 choose_a_winner 会在 Switchboard reveal 成功后：
    // 1. 确认传入的是 commit 阶段锁定的 Randomness Account；
    // 2. 检查 authority、彩票结束 slot 和是否已经开奖；
    // 3. 读取已 reveal 的随机值；
    // 4. random % ticket_num 得到中奖票号，并写入 token_lottery。
    const revealIx = await program.methods
      .chooseAWinner()
      .accounts({
        randomnessAccountData: randomness.pubkey,
      })
      .instruction();

    // 指令顺序同样不能交换：Switchboard 先验证 Oracle 签名并写入随机值，
    // Lottery 随后在同一交易中读取这个值并选出 winner。
    const revealTx = await sb.asV0Tx({
      connection: switchboardProgram.provider.connection,
      ixs: [sbRevealIx, revealIx],
      payer: wallet.publicKey,
      signers: [wallet.payer],
      computeUnitPrice: 75_000,
      computeUnitLimitMultiple: 1.3,
    });

    const revealSignature = await connection.sendTransaction(revealTx);
    // 必须等待 reveal 交易确认后再进入下一个 claim 测试。这里若误用上面的
    // commitSignature，只会再次确认旧交易，claim 可能抢在 winner 写入前执行，
    // 最终报 WinnerNotChosen。
    await connection.confirmTransaction({
      signature: revealSignature,
      blockhash: blockhashContext.value.blockhash,
      lastValidBlockHeight: blockhashContext.value.lastValidBlockHeight,
    });
    console.log("✅ Transaction Signature for reveal:", revealSignature);
  });

  it("Is claiming a prize", async () => {
    // return
    // 彩票状态 PDA，必须与 Rust 中 seeds=[b"token_lottery"] 完全一致。
    const tokenLotteryAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("token_lottery")],
      program.programId
    )[0];
    // 读取开奖后的链上状态，主要关心 winner、winnerChosen 和奖池金额。
    const lotteryConfig = await program.account.tokenLottery.fetch(
      tokenLotteryAddress
    );
    console.log("Lottery winner", lotteryConfig.winner);
    console.log("Lottery config", lotteryConfig);

    // 打印测试钱包持有的全部 SPL Token Account，方便观察前面购买的 5 张票。
    const tokenAccounts = await connection.getParsedTokenAccountsByOwner(
      wallet.publicKey,
      { programId: TOKEN_PROGRAM_ID }
    );
    tokenAccounts.value.forEach(async (account) => {
      console.log("Token account mint", account.account.data.parsed.info.mint);
      console.log("Token account address", account.pubkey.toBase58());
    });

    // 买票时 Rust 使用 ticket_num.to_le_bytes() 作为 Ticket Mint PDA seed。
    // winner 就是中奖 ticket_num，所以这里也必须转成“8 字节小端序”。
    // 如果前端错误使用大端序或不同长度，推导出的 Mint 地址会完全不同。
    const winningMint = anchor.web3.PublicKey.findProgramAddressSync(
      [new anchor.BN(lotteryConfig.winner).toArrayLike(Buffer, "le", 8)],
      program.programId
    )[0];
    console.log("Winning mint", winningMint.toBase58());

    // 推导测试钱包中用于保存中奖 NFT 的 ATA。Rust 端还会验证：
    // ATA 的 owner 是 payer、mint 是 winningMint、余额大于 0，并且该 NFT 的
    // Metadata 属于已经验证的 Lottery Collection。
    const winningTokenAddress = getAssociatedTokenAddressSync(
      winningMint,
      wallet.publicKey
    );
    console.log("Winning token address", winningTokenAddress.toBase58());

    // Anchor 会根据 winner、PDA seeds 和 associated_token 账户约束自动补齐
    // ticketMint、metadata、collectionMint、destination 等账户。
    const claimIx = await program.methods
      .claimPrize()
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    const blockhashContext = await connection.getLatestBlockhash();

    const claimTx = new anchor.web3.Transaction({
      blockhash: blockhashContext.blockhash,
      lastValidBlockHeight: blockhashContext.lastValidBlockHeight,
      feePayer: wallet.payer.publicKey,
    }).add(claimIx);

    // claim_prize 校验通过后，把 lottery_pot_amount 对应的 lamports 从
    // token_lottery PDA 转给 payer，然后把 lottery_pot_amount 清零。
    const claimSig = await anchor.web3.sendAndConfirmTransaction(
      connection,
      claimTx,
      [wallet.payer]
    );
    console.log(claimSig);
  });
});
