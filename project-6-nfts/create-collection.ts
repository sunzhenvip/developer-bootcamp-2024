// 从 Metaplex 的 mpl-token-metadata 库中导入函数：
// createNft - 创建 NFT
// fetchDigitalAsset - 查询 NFT 数据
// mplTokenMetadata - 加载 Token Metadata 插件
import {
  createNft,
  fetchDigitalAsset,
  mplTokenMetadata,
} from "@metaplex-foundation/mpl-token-metadata";
import { base58 } from "@metaplex-foundation/umi-serializers";
// 从 Solana Helpers 工具库导入：
// airdropIfRequired - 如果余额不足则空投 SOL
// getExplorerLink - 生成 Solana Explorer 浏览器链接
// getKeypairFromFile - 从本地密钥文件读取 keypair
import {
  airdropIfRequired,
  getExplorerLink,
  getKeypairFromFile,
} from "@solana-developers/helpers";
// Umi 是 Metaplex 的新版框架，用于处理链上事务
// createUmi - 创建 Umi 客户端实例
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
// Solana Web3.js 库
// Connection - RPC 连接
// LAMPORTS_PER_SOL - 单位转换 (1 SOL = 10^9 Lamports)
// clusterApiUrl - 获取指定集群 RPC 地址
import { Connection, LAMPORTS_PER_SOL, clusterApiUrl } from "@solana/web3.js";
// Umi 内置工具：
// generateSigner - 生成新的随机 Keypair (钱包)
// keypairIdentity - 设置钱包身份，用于签名交易
// percentAmount - 百分比工具函数（这里用来设置版税）
import {
  generateSigner,
  keypairIdentity,
  percentAmount,
} from "@metaplex-foundation/umi";
// 建立与 Devnet 的 RPC 连接
const connection = new Connection(clusterApiUrl("devnet"));
// 从本地文件读取用户钱包 keypair (默认为 ~/.config/solana/id.json)
const user = await getKeypairFromFile();

// 如果余额不足 1 SOL，则自动空投 0.5 SOL，保证有手续费可以用
await airdropIfRequired(
  connection,
  user.publicKey,
  1 * LAMPORTS_PER_SOL,
  0.5 * LAMPORTS_PER_SOL
);

console.log("user token 地址 ", user.publicKey.toString());
// 查询余额（单位：lamports）
const balanceLamports = await connection.getBalance(user.publicKey);
// 转换为 SOL
const balanceSol = balanceLamports / LAMPORTS_PER_SOL;
console.log(`余额: ${balanceSol} SOL`);


// 创建 Umi 客户端实例，绑定到 Devnet 的 RPC
const umi = createUmi(connection.rpcEndpoint);
// 加载 Token Metadata 插件，支持 NFT 元数据操作
umi.use(mplTokenMetadata());
// 用现有的 Solana Keypair 创建一个 Umi Keypair
const umiUser = umi.eddsa.createKeypairFromSecretKey(user.secretKey);
// 设置用户身份（签名用的钱包）为 umiUser
umi.use(keypairIdentity(umiUser));

console.log("Set up Umi instance for user => 为用户设置Umi实例");

// 生成一个新的 Mint 地址 (NFT 的唯一标识)
const collectionMint = generateSigner(umi);

// 构造一笔交易，创建一个 Collection NFT
const transaction = await createNft(umi, {
  mint: collectionMint, // NFT 的 mint 地址
  name: "My Collection", // NFT 名称
  symbol: "MC", // NFT 符号
  uri: "https://raw.githubusercontent.com/solana-developers/professional-education/main/labs/sample-nft-collection-offchain-data.json",
  sellerFeeBasisPoints: percentAmount(0), // 版税 = 0%
  isCollection: true, // 标记为 Collection NFT
});
// 发送并确认交易
// 第二个参数 ,{ send: { commitment: "finalized" } }
let result = await transaction.sendAndConfirm(umi);
const txSignature = base58.deserialize(result.signature)[0];
console.log("signature ", txSignature);
console.log("collectionMint ", collectionMint.publicKey.toString());
console.log(`稍等 10 秒钟加载 获取链上数据.....`);
await sleep(10_000); // 等待 10000 毫秒 = 2 秒
// 从链上获取刚刚创建的 NFT 资产信息
const createdCollectionNft = await fetchDigitalAsset(
  umi,
  collectionMint.publicKey
);
// 打印 Explorer 浏览器地址，方便查看 NFT
console.log(
  `Created Collection 📦! Address is ${getExplorerLink(
    "address",
    createdCollectionNft.mint.publicKey,
    "devnet"
  )}`
);

/*
user token 地址  sunkq6xnqcHMqrrBX7wPTHS9sVCZ9SNTentvHyFSnxh
余额: 9.52824256 SOL
Set up Umi instance for user => 为用户设置Umi实例
signature  3qAw6jRTfkJXKrfoJDeCoZgXCU5WdidgFp8yKj7B3FH8ga2a6CC6gxuJLfzLLDZTKE9t3rs8JZ6DrrY3NGg2Y5ks
collectionMint  3nCgXqMjBQt52xvjPzwrBYhAccrx9iK7sqnsmNeFLEeA
稍等 10 秒钟加载 获取链上数据.....
Created Collection 📦! Address is https://explorer.solana.com/address/3nCgXqMjBQt52xvjPzwrBYhAccrx9iK7sqnsmNeFLEeA?cluster=devnet
*/


// 等待指定毫秒数
function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}
