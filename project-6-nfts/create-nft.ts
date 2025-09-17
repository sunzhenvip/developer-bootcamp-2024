import {
  createNft,
  fetchDigitalAsset,
  mplTokenMetadata,
} from "@metaplex-foundation/mpl-token-metadata";

import {
  airdropIfRequired,
  getExplorerLink,
  getKeypairFromFile,
} from "@solana-developers/helpers";

import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";

import { Connection, LAMPORTS_PER_SOL, clusterApiUrl } from "@solana/web3.js";
import {
  generateSigner,
  keypairIdentity,
  percentAmount,
  publicKey,
} from "@metaplex-foundation/umi";
import {base58} from "@metaplex-foundation/umi-serializers";

const connection = new Connection(clusterApiUrl("devnet"));

const user = await getKeypairFromFile();

await airdropIfRequired(
  connection,
  user.publicKey,
  1 * LAMPORTS_PER_SOL,
  0.5 * LAMPORTS_PER_SOL
);

console.log("Loaded user", user.publicKey.toBase58());

const umi = createUmi(connection.rpcEndpoint);
umi.use(mplTokenMetadata());

const umiUser = umi.eddsa.createKeypairFromSecretKey(user.secretKey);
umi.use(keypairIdentity(umiUser));

console.log("Set up Umi instance for user");

const collectionAddress = publicKey(
  "7h3DckfS79EtKvWUonEbwEsaetXc7uj6QWUt9TELbrg4"
);

console.log(`Creating NFT...`);

const mint = generateSigner(umi);

const transaction = await createNft(umi, {
  mint,
  name: "My NFT",
  uri: "https://raw.githubusercontent.com/sunzhenvip/developer-bootcamp-2024/refs/heads/master/project-6-nfts/nft-offchain-data.json",
  sellerFeeBasisPoints: percentAmount(0),
  collection: {
    key: collectionAddress,
    verified: false,
  },
});

let result = await transaction.sendAndConfirm(umi);

const txSignature = base58.deserialize(result.signature)[0];
console.log("createNft signature ", txSignature);
console.log("mint ", mint.publicKey.toString()); // F3CQRR9XDNASMxt4YcRBqHWxLJFE5hYxu3KWvCKXx55P
console.log(`稍等 20 秒钟加载 获取链上数据.....`);
await sleep(20_000); // 等待 10000 毫秒 = 2 秒

const createdNft = await fetchDigitalAsset(umi, mint.publicKey);

console.log(
  `🖼️ Created NFT! Address is ${getExplorerLink(
    "address",
    createdNft.mint.publicKey,
    "devnet"
  )}`
); // 🖼️ Created NFT! Address is https://explorer.solana.com/address/F3CQRR9XDNASMxt4YcRBqHWxLJFE5hYxu3KWvCKXx55P?cluster=devnet


function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}
