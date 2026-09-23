import * as anchor from "@coral-xyz/anchor";
import * as sb from "@switchboard-xyz/on-demand";
import { Program } from "@coral-xyz/anchor";
import { TokenLottery } from "../target/types/token_lottery";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { Idl } from "@coral-xyz/anchor/dist/cjs/idl";
describe("token-lottery", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenLottery as Program<TokenLottery>;
  let switchboardProgram: Program<any>;
  let metaDataProgramLength: any;
  const rngKp = anchor.web3.Keypair.generate();

  const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
  );
  const apiKey = "c5730fdb-3471-42ff-92ad-97256fa83871";
  async function getSwitchboardIDL() {
    const switchboardIDL: Idl | null = await anchor.Program.fetchIdl(
      sb.ON_DEMAND_MAINNET_PID, // sb.SB_ON_DEMAND_PID,一开始是这个应该是写错了
      {
        connection: new anchor.web3.Connection(
          "https://mainnet.helius-rpc.com/?api-key=" + apiKey
        ),
      }
    );
    // 在使用前进行空值检查
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
    const switchboardIDL = require("../tests/switchboard-idl.json"); // 本地 IDL 文件
    switchboardProgram = new anchor.Program(switchboardIDL, provider);

    const accountInfo = await connection.getAccountInfo(
      TOKEN_METADATA_PROGRAM_ID
    );
    metaDataProgramLength = accountInfo?.data.length;
  });

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
  async function buyTicket() {
    const buyTicketIx = await program.methods
      .buyTicket()
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    const blockhashContext = await connection.getLatestBlockhash();

    const computeIx = anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 300000,
    });

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

    const sig = await anchor.web3.sendAndConfirmTransaction(connection, tx, [
      wallet.payer,
    ]);
    console.log("buy ticket ", sig);
  }

  it("Is initialized!", async () => {
    const slot = await connection.getSlot();
    console.log("Current slot", slot);

    const mint = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("collection_mint")],
      program.programId
    )[0];

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

    const initConfigIx = await program.methods
      .initializeConfig(
        new anchor.BN(0),
        new anchor.BN(slot + 10),
        new anchor.BN(10000)
      )
      .instruction();

    const initLotteryIx = await program.methods
      .initializeLottery()
      .accounts({
        masterEdition: masterEdition,
        metadata: metadata,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    const blockhashContext = await connection.getLatestBlockhash();

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
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();
  });

  it("Is committing and revealing a winner", async () => {
    const queue_addr = "A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w"; // switchboard
    const queue = new anchor.web3.PublicKey(
      "A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w"
    );

    const queueAccount = new sb.Queue(switchboardProgram, queue);
    console.log("Queue account", queue.toString());
    try {
      const loadData = await queueAccount.loadData();
      console.log("await queueAccount.loadData() ", loadData.oracleKeys.length);
      /*for (let i = 0; i < loadData.oracleKeys.length; i++) {
        console.log(loadData.oracleKeys[i].toString())
      }*/
    } catch (err) {
      console.error("❌ Queue account not found:", err);
      process.exit(1);
    }

    const [randomness, ix] = await sb.Randomness.create(
      switchboardProgram,
      rngKp,
      queue
    );
    console.log("Created randomness account..");
    console.log("Randomness account", randomness.pubkey.toBase58());
    console.log("rkp account", rngKp.publicKey.toBase58());
    const createRandomnessTx = await sb.asV0Tx({
      connection: connection,
      ixs: [ix],
      payer: wallet.publicKey,
      signers: [wallet.payer, rngKp],
      computeUnitPrice: 75_000,
      computeUnitLimitMultiple: 1.3,
    });

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
    const sbCommitIx = await randomness.commitIx(queue); // 这一行报错
    console.log("sbCommitIx", sbCommitIx.programId.toString());
    const commitIx = await program.methods
      .commitAWinner()
      .accounts({
        randomnessAccountData: randomness.pubkey,
      })
      .instruction();

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
    const sbRevealIx = await randomness.revealIx();
    const revealIx = await program.methods
      .chooseAWinner()
      .accounts({
        randomnessAccountData: randomness.pubkey,
      })
      .instruction();

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
    const tokenLotteryAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("token_lottery")],
      program.programId
    )[0];
    const lotteryConfig = await program.account.tokenLottery.fetch(
      tokenLotteryAddress
    );
    console.log("Lottery winner", lotteryConfig.winner);
    console.log("Lottery config", lotteryConfig);

    const tokenAccounts = await connection.getParsedTokenAccountsByOwner(
      wallet.publicKey,
      { programId: TOKEN_PROGRAM_ID }
    );
    tokenAccounts.value.forEach(async (account) => {
      console.log("Token account mint", account.account.data.parsed.info.mint);
      console.log("Token account address", account.pubkey.toBase58());
    });

    const winningMint = anchor.web3.PublicKey.findProgramAddressSync(
      [new anchor.BN(lotteryConfig.winner).toArrayLike(Buffer, "le", 8)],
      program.programId
    )[0];
    console.log("Winning mint", winningMint.toBase58());

    const winningTokenAddress = getAssociatedTokenAddressSync(
      winningMint,
      wallet.publicKey
    );
    console.log("Winning token address", winningTokenAddress.toBase58());

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

    const claimSig = await anchor.web3.sendAndConfirmTransaction(
      connection,
      claimTx,
      [wallet.payer]
    );
    console.log(claimSig);
  });
});
