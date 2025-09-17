import {getAssociatedTokenAddress, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID} from "@solana/spl-token";
import {PublicKey} from "@solana/web3.js";

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