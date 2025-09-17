// This uses "@metaplex-foundation/mpl-token-metadata@2" to create tokens
import "dotenv/config";
import {
  getKeypairFromEnvironment,
  getExplorerLink, getKeypairFromFile,
} from "@solana-developers/helpers";
import {
  Connection,
  clusterApiUrl,
  PublicKey,
  Transaction,
  sendAndConfirmTransaction,
  LAMPORTS_PER_SOL
} from "@solana/web3.js";
import {
  DataV2,
  createCreateMetadataAccountV3Instruction,
} from "@metaplex-foundation/mpl-token-metadata";

// const user = getKeypairFromEnvironment("SECRET_KEY"); // 读取 .env文件 环境变量 SECRET_KEY
const user = await getKeypairFromFile(); // 默认读取 ~/.config/solana/id.json 文件

const connection = new Connection(clusterApiUrl("devnet"));

console.log(
  `🔑 We've loaded our keypair securely, using an env file! Our public key is: ${user.publicKey.toBase58()}`
);

// 查询余额（单位：lamports）
const balanceLamports = await connection.getBalance(user.publicKey);
// 转换为 SOL
const balanceSol = balanceLamports / LAMPORTS_PER_SOL;
console.log(`余额: ${balanceSol} SOL`);

const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

// Subtitute in your token mint account
const tokenMintAccount = new PublicKey("tescD95ij1c27G7mDHXzN8wY3M6AuzN7nv6F5u813RV");

const tokenMintLink0 = getExplorerLink(
    "address",
    tokenMintAccount.toString(),
    "devnet"
);

console.log(`✅ Look at the token mint again: ${tokenMintLink0}`);

const metadataData: DataV2 = {
  name: "Solana Training Token",
  symbol: "TRAINING",
  // An off-chain link to more information about the token using Metaplex standard for off-chain data
  // We are using a GitHub link here, but in production this content would be hosted on an immutable storage like
  // Arweave / IPFS / Pinata etc
  uri: "https://raw.githubusercontent.com/solana-developers/professional-education/main/labs/sample-token-metadata.json",
  sellerFeeBasisPoints: 0,
  creators: null,
  collection: null,
  uses: null,
};

const metadataPDAAndBump = PublicKey.findProgramAddressSync(
  [
    Buffer.from("metadata"),
    TOKEN_METADATA_PROGRAM_ID.toBuffer(),
    tokenMintAccount.toBuffer(),
  ],
  TOKEN_METADATA_PROGRAM_ID
);

const metadataPDA = metadataPDAAndBump[0];

const transaction = new Transaction();

const createMetadataAccountInstruction =
  createCreateMetadataAccountV3Instruction(
    {
      metadata: metadataPDA,
      mint: tokenMintAccount,
      mintAuthority: user.publicKey,
      payer: user.publicKey,
      updateAuthority: user.publicKey,
    },
    {
      createMetadataAccountArgsV3: {
        collectionDetails: null,
        data: metadataData,
        isMutable: true,
      },
    }
  );

transaction.add(createMetadataAccountInstruction);

const transactionSignature = await sendAndConfirmTransaction( // 这里出错
  connection,
  transaction,
  [user]
);

const transactionLink = getExplorerLink(
  "transaction",
  transactionSignature,
  "devnet"
);

console.log(`✅ Transaction confirmed, explorer link is: ${transactionLink}!`);

const tokenMintLink = getExplorerLink(
  "address",
  tokenMintAccount.toString(),
  "devnet"
);

console.log(`✅ Look at the token mint again: ${tokenMintLink}!`);
