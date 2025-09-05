import * as anchor from "@coral-xyz/anchor";
import * as sb from "@switchboard-xyz/on-demand";
import { Program } from "@coral-xyz/anchor";
import { TokenLottery } from "../target/types/token_lottery";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";

async function switchboardRandomness() {
  const apiKey = "0738aea2-3950-43f9-85fd-81876b66f752";
  const url = "https://mainnet.helius-rpc.com/?api-key=" + apiKey;
  // const connection = new anchor.web3.Connection(url, "confirmed");
  const connection = new anchor.web3.Connection("http://127.0.0.1:8899");
  const keypair = await sb.AnchorUtils.initKeypairFromFile("/home/sz/.config/solana/id.json");
  const wallet = new NodeWallet(keypair);
  const provider = new anchor.AnchorProvider(connection,wallet)
  const pid = sb.ON_DEMAND_MAINNET_PID;
  // const pid1 = sb.SB_ON_DEMAND_PID;
  const program = await anchor.Program.at(pid, provider);
  console.log("\nSetup...");
  console.log("Program", program!.programId.toString());

  const sbQueue = new anchor.web3.PublicKey("A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w");
  const queueAccount = new sb.Queue(program, sbQueue);
  // 加载数据
  const queueData = await queueAccount.loadData();
  // console.log("queueData",queueData);
  // 1. 生成随机数账户 & 初始化指令
  // const rngKp = anchor.web3.Keypair.generate();
  const rngKp = keypair;
  console.log("rngKp", rngKp.publicKey.toString());
  const [randomness, initIx] = await sb.Randomness.create(program, rngKp, sbQueue);

  console.log("Randomness pubkey:", randomness.pubkey.toString());
  // 2. 先发送交易创建 Randomness 账户
  let tx = new anchor.web3.Transaction().add(initIx);
  let sig = await provider.sendAndConfirm(tx, [rngKp]);
  console.log("Init TX:", sig);


  // 3. 再调用 commitIx（提交随机数请求）
  const commitIx = await randomness.commitIx(sbQueue);
  tx = new anchor.web3.Transaction().add(commitIx);
  sig = await provider.sendAndConfirm(tx);
  console.log("Commit TX:", sig);
}


describe("token-lottery", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenLottery as Program<TokenLottery>;
  let switchboardProgram;
  let metaDataProgramLength;
  const rngKp = anchor.web3.Keypair.generate();

  const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');
  const apiKey = "c5730fdb-3471-42ff-92ad-97256fa83871";

  // 没有 api-key 可以在这个网站注册获取一个 免费的 有速率限制 每秒钟几个 https://www.helius.dev/
  before("Loading switchboard program", async () => {
    /*const switchboardIDL = await anchor.Program.fetchIdl(
      sb.ON_DEMAND_MAINNET_PID, // sb.SB_ON_DEMAND_PID,一开始是这个应该是写错了
      {connection: new anchor.web3.Connection("https://mainnet.helius-rpc.com/?api-key=" + apiKey)}
    );
    switchboardProgram = new anchor.Program(switchboardIDL, provider);*/
    /*var fs = require('fs');
    fs.writeFile('tests/switchboard-idl.json', JSON.stringify(switchboardIDL), function (err) {
      if (err) throw err;
      console.log('The file has been saved!');
    })*/

    const switchboardIDL = require("../tests/switchboard-idl.json"); // 本地 IDL 文件
    switchboardProgram = new anchor.Program(switchboardIDL,provider);

    const accountInfo = await connection.getAccountInfo(TOKEN_METADATA_PROGRAM_ID);
    metaDataProgramLength = accountInfo?.data.length
  });

  it("测试是否正常获取数据", async () => {
    console.log("ondemand.so 合约公钥地址", switchboardProgram.programId.toString());
    console.log("metadata.so 账户存储字节", metaDataProgramLength);
  })
  // console.log("已退出");
  // return
  async function buyTicket() {
    const buyTicketIx = await program.methods.buyTicket()
      .accounts({
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .instruction();

    const blockhashContext = await connection.getLatestBlockhash();

    const computeIx = anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
     units: 300000
    });

    const priorityIx = anchor.web3.ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: 1
    });

    const tx = new anchor.web3.Transaction({
      blockhash: blockhashContext.blockhash,
      lastValidBlockHeight: blockhashContext.lastValidBlockHeight,
      feePayer: wallet.payer.publicKey,
    }).add(buyTicketIx)
      .add(computeIx)
      .add(priorityIx);

    const sig = await anchor.web3.sendAndConfirmTransaction(connection, tx, [wallet.payer]);
    console.log("buy ticket ", sig);
  }

  it("Is initialized!", async () => {

    const slot = await connection.getSlot();
    console.log("Current slot", slot);

    const mint = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('collection_mint')],
      program.programId,
    )[0];

    const metadata = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
        TOKEN_METADATA_PROGRAM_ID,
      )[0];
  
    const masterEdition = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from('metadata'), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from('edition')],
        TOKEN_METADATA_PROGRAM_ID,
      )[0];

    const initConfigIx = await program.methods.initializeConfig(
      new anchor.BN(0),
      new anchor.BN(slot + 10),
      new anchor.BN(10000),
    ).instruction();

    const initLotteryIx = await program.methods.initializeLottery()
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
    }).add(initConfigIx)
      .add(initLotteryIx);

    const sig = await anchor.web3.sendAndConfirmTransaction(connection, tx, [wallet.payer]);
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
    const queue = new anchor.web3.PublicKey("A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w");

    const queueAccount = new sb.Queue(switchboardProgram, queue);
    console.log("Queue account", queue.toString());
    try {
      await queueAccount.loadData();
    } catch (err) {
      console.error("❌ Queue account not found:", err);
      process.exit(1);
    }

    const [randomness, ix] = await sb.Randomness.create(switchboardProgram, rngKp, queue);
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
  
    const createRandomnessSignature = await connection.sendTransaction(createRandomnessTx);
    await connection.confirmTransaction({
      signature: createRandomnessSignature,
      blockhash: blockhashContext.value.blockhash,
      lastValidBlockHeight: blockhashContext.value.lastValidBlockHeight
    });
    console.log(
      "Transaction Signature for randomness account creation: ",
      createRandomnessSignature
    );
    const queueData = await randomness.loadData();
    console.log("Queue data", queueData.authority.toString());
    const sbCommitIx = await randomness.commitIx(queue);
    console.log("sbCommitIx",sbCommitIx.programId.toString());
    const commitIx = await program.methods.commitAWinner()
      .accounts(
        {
          randomnessAccountData: randomness.pubkey
        }
      )
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
      lastValidBlockHeight: blockhashContext.value.lastValidBlockHeight
    });
    console.log(
      "Transaction Signature for commit: ",
      commitSignature
    );

    const sbRevealIx = await randomness.revealIx();
    const revealIx = await program.methods.chooseAWinner()
      .accounts({
        randomnessAccountData: randomness.pubkey
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
    await connection.confirmTransaction({
      signature: commitSignature,
      blockhash: blockhashContext.value.blockhash,
      lastValidBlockHeight: blockhashContext.value.lastValidBlockHeight
    });
    console.log("✅ Transaction Signature for reveal:", revealSignature);
  });

  it("Is claiming a prize", async () => {
    return
    const tokenLotteryAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('token_lottery')],
      program.programId,
    )[0];
    const lotteryConfig = await program.account.tokenLottery.fetch(tokenLotteryAddress);
    console.log("Lottery winner", lotteryConfig.winner);
    console.log("Lottery config", lotteryConfig);


    const tokenAccounts = await connection.getParsedTokenAccountsByOwner(wallet.publicKey, {programId: TOKEN_PROGRAM_ID});
    tokenAccounts.value.forEach(async (account) => {
      console.log("Token account mint", account.account.data.parsed.info.mint);
      console.log("Token account address", account.pubkey.toBase58());
    });

    const winningMint = anchor.web3.PublicKey.findProgramAddressSync(
      [new anchor.BN(lotteryConfig.winner).toArrayLike(Buffer, 'le', 8)],
      program.programId,
    )[0];
    console.log("Winning mint", winningMint.toBase58());

    const winningTokenAddress = getAssociatedTokenAddressSync(
      winningMint,
      wallet.publicKey
    );
    console.log("Winning token address", winningTokenAddress.toBase58());

    const claimIx = await program.methods.claimPrize()
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

    const claimSig = await anchor.web3.sendAndConfirmTransaction(connection, claimTx, [wallet.payer]);
    console.log(claimSig);

  });

});







/*
  it("Is committing and revealing a winner", async () => {
    // await switchboardRandomness();
    // return
    // Switchboard Queue
    // FfD96yeXs4cxZshoPPSKhSPgVQxLAJUT3gefgh84m1Di
    // A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w 原来的
    const queue = new anchor.web3.PublicKey(
      "A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w"
    );
    const queueAccount = new sb.Queue(switchboardProgram, queue);
    console.log("Queue account", queue.toBase58());
    try {
      await queueAccount.loadData();
    } catch (err) {
      console.error("❌ Queue account not found:", err);
      process.exit(1);
    }

    // 1. 创建 randomness account
    const [randomness, ix] = await sb.Randomness.create(
      switchboardProgram,
      rngKp,
      queue
    );

    console.log("Created randomness account..");
    console.log("Randomness account", randomness.pubkey.toBase58());
    console.log("rkp account", rngKp.publicKey.toBase58());
    const createRandomnessTx = await sb.asV0Tx({
      connection,
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
      "✅ Transaction Signature for randomness account creation:",
      createRandomnessSignature
    );


    // 2. 加载 randomness 数据
    const queueData = await randomness.loadData();

    console.log("Queue data", queueData.authority.toString());
    // return
    console.log("xxxxxxxxx",createRandomnessSignature);
    // 3. Commit winner
    const sbCommitIx = await randomness.commitIx(queue);
    console.log("总是出错的这一行数 sbCommitIxxxxxxxxx", sbCommitIx.programId);
    // return
    const commitIx = await program.methods
      .commitAWinner()
      .accounts({
        randomnessAccountData: randomness.pubkey,
      })
      .instruction();

    const commitTx = await sb.asV0Tx({
      connection,
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

    console.log("✅ Transaction Signature for commit:", commitSignature);

    // 4. Reveal winner
    const sbRevealIx = await randomness.revealIx();
    const revealIx = await program.methods
      .chooseAWinner()
      .accounts({
        randomnessAccountData: randomness.pubkey,
      })
      .instruction();

    const revealTx = await sb.asV0Tx({
      connection,
      ixs: [sbRevealIx, revealIx],
      payer: wallet.publicKey,
      signers: [wallet.payer],
      computeUnitPrice: 75_000,
      computeUnitLimitMultiple: 1.3,
    });

    const revealSignature = await connection.sendTransaction(revealTx);
    await connection.confirmTransaction({
      signature: revealSignature,
      blockhash: blockhashContext.value.blockhash,
      lastValidBlockHeight: blockhashContext.value.lastValidBlockHeight,
    });

    console.log("✅ Transaction Signature for reveal:", revealSignature);
  });
*/