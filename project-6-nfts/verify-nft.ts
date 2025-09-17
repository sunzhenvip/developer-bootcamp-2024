import {
  findMetadataPda,
  mplTokenMetadata,
  verifyCollectionV1,
} from "@metaplex-foundation/mpl-token-metadata";

import {
  airdropIfRequired,
  getExplorerLink,
  getKeypairFromFile,
} from "@solana-developers/helpers";

import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";

import { Connection, LAMPORTS_PER_SOL, clusterApiUrl } from "@solana/web3.js";
import { keypairIdentity, publicKey } from "@metaplex-foundation/umi";
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

// We could also dp
const collectionAddress = publicKey(
  "7h3DckfS79EtKvWUonEbwEsaetXc7uj6QWUt9TELbrg4"
);

const nftAddress = publicKey("F3CQRR9XDNASMxt4YcRBqHWxLJFE5hYxu3KWvCKXx55P");

const transaction = await verifyCollectionV1(umi, {
  metadata: findMetadataPda(umi, { mint: nftAddress }),
  collectionMint: collectionAddress,
  authority: umi.identity,
});

let result = await transaction.sendAndConfirm(umi);

const txSignature = base58.deserialize(result.signature)[0];
console.log("verifyCollectionV1 signature ", txSignature);

console.log(`稍等 20 秒钟加载 获取链上数据.....`);
await sleep(20_000); // 等待 10000 毫秒 = 2 秒

console.log(
  `✅ NFT ${nftAddress} verified as member of collection ${collectionAddress}! See Explorer at ${getExplorerLink(
    "address",
    nftAddress,
    "devnet"
  )}`
);

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}
