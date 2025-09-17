import { createMint } from "@solana/spl-token";
import "dotenv/config";
import {
  getKeypairFromEnvironment,
  getExplorerLink,
  getKeypairFromFile,
} from "@solana-developers/helpers";
import {Connection, clusterApiUrl, LAMPORTS_PER_SOL,} from "@solana/web3.js";

const connection = new Connection(clusterApiUrl("devnet"));

// const user = getKeypairFromEnvironment("SECRET_KEY");
const user = await getKeypairFromFile(); // 默认读取 ~/.config/solana/id.json 文件

console.log(
  `🔑 Loaded our keypair securely, using an env file! Our public key is: ${user.publicKey.toBase58()}`
);

const balanceLamports = await connection.getBalance(user.publicKey);
// 转换为 SOL
const balanceSol = balanceLamports / LAMPORTS_PER_SOL;
console.log(`余额: ${balanceSol} SOL`);

// This is a shortcut that runs:
// SystemProgram.createAccount
// token.createInitializeMintInstruction
// See https://www.soldev.app/course/token-program
const tokenMint = await createMint(connection, user, user.publicKey, null, 2);

console.log("tokenMint", tokenMint); // 94tfu4LVsTY1Xwb9RS7Cebhnma1oSTDBM5Xqpz2Pz1xS

const link = getExplorerLink("address", tokenMint.toString(), "devnet");

console.log(`✅ Success! Created token mint: ${link}`);

// ✅ Success! Created token mint: https://explorer.solana.com/address/94tfu4LVsTY1Xwb9RS7Cebhnma1oSTDBM5Xqpz2Pz1xS?cluster=devnet
