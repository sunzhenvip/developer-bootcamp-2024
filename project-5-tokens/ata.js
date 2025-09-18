import {
    getAccount,
    getAssociatedTokenAddress,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import {
    PublicKey,
    Connection,
    clusterApiUrl,
    Transaction,
    sendAndConfirmTransaction,
    LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {getKeypairFromFile} from "@solana-developers/helpers";

const payer = await getKeypairFromFile(); // 默认读取 ~/.config/solana/id.json 文件

const connection = new Connection(clusterApiUrl("devnet"));

console.log(`🔑 public key is: ${payer.publicKey.toBase58()}`);

const balanceLamports = await connection.getBalance(payer.publicKey);
const balanceSol = balanceLamports / LAMPORTS_PER_SOL;
console.log(`余额: ${balanceSol} SOL`);

const mint = new PublicKey("minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG");
const owner = new PublicKey("sunkq6xnqcHMqrrBX7wPTHS9sVCZ9SNTentvHyFSnxh");
// 第一个种方式获取ATA
const ata1Address = await getAssociatedTokenAddress(mint, owner, false, TOKEN_2022_PROGRAM_ID);
console.log("ATA:", ata1Address.toBase58());

// 第二种方式获取ATA
const [ata2Address, bumpSeed] = PublicKey.findProgramAddressSync(
    [owner.toBuffer(), TOKEN_2022_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID
)
console.log("ATA:", ata2Address.toBase58());
console.log("ATA 的 bumpSeed", bumpSeed);


// spl-token account-info --program-id TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb  minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG

const tokenAccountPubkey = new PublicKey("minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG");

const tokenAccount1 = await getAccount(connection, ata1Address, "confirmed", TOKEN_2022_PROGRAM_ID);

console.log(tokenAccount1);


// const info = await connection.getAccountInfo(tokenAccountPubkey, "confirmed");
// console.log(info.toString());

// 2CRa9yBXR3BDgGN2dup66USNoMrRk1W8J5QBaEApAMpo
// 2CRa9yBXR3BDgGN2dup66USNoMrRk1W8J5QBaEApAMpo