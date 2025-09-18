import {
    Connection,
    Keypair,
    SystemProgram,
    Transaction,
    clusterApiUrl,
    sendAndConfirmTransaction, PublicKey, LAMPORTS_PER_SOL,ComputeBudgetProgram
} from "@solana/web3.js";
import {
    ExtensionType,
    TOKEN_2022_PROGRAM_ID,
    createInitializeMintInstruction,
    getMintLen,
    createInitializeMetadataPointerInstruction,
    getMint,
    getMetadataPointerState,
    getTokenMetadata,
    TYPE_SIZE,
    LENGTH_SIZE,
    getAssociatedTokenAddress,
    createAssociatedTokenAccount, ASSOCIATED_TOKEN_PROGRAM_ID,
    createAssociatedTokenAccountInstruction
} from "@solana/spl-token";
import {
    createInitializeInstruction,
    createUpdateFieldInstruction,
    createRemoveKeyInstruction,
    pack,
    TokenMetadata,
} from "@solana/spl-token-metadata";
import {getKeypairFromFile} from "@solana-developers/helpers";

// Playground wallet
const payer = await getKeypairFromFile();

// Connection to devnet cluster
const connection = new Connection(clusterApiUrl("devnet"), "confirmed");

console.log("user token 地址 ", payer.publicKey.toString());
// 查询余额（单位：lamports）
const balanceLamports = await connection.getBalance(payer.publicKey);
// 转换为 SOL
const balanceSol = balanceLamports / LAMPORTS_PER_SOL;
console.log(`余额: ${balanceSol} SOL`);
// Transaction to send
let transaction: Transaction;
// Transaction signature returned from sent transaction
let transactionSignature: string;

// get file from local
const mintFileName = "../project-5-tokens/tescD95ij1c27G7mDHXzN8wY3M6AuzN7nv6F5u813RV.json";
const mintKeypair = await getKeypairFromFile(mintFileName);
// Address for Mint Account
// 提前执行了这个命令 链上运行了
// #1 - System Program: createAccount
// #2 - Token 2022 Program: initializeMetadataPointer
// #3 - Token 2022 Program: initializeMint
// #4 - Compute Budget: SetComputeUnitLimit
// spl-token create-token --program-id TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb --enable-metadata tescD95ij1c27G7mDHXzN8wY3M6AuzN7nv6F5u813RV.json
const mint = mintKeypair.publicKey; // 已经通过命令创建
// Decimals for Mint Account
const decimals = 2;
// Authority that can mint new tokens
const mintAuthority = payer.publicKey;
// Authority that can update token metadata
const updateAuthority = payer.publicKey;

// Metadata to store in Mint Account
const metaData: TokenMetadata = {
    updateAuthority: updateAuthority,
    mint: mint,
    name: "OPOS",
    symbol: "OPOS",
    uri: "https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/metadata.json",
    additionalMetadata: [["description", "Only Possible On Solana"]],
};

// Size of MetadataExtension 2 bytes for type, 2 bytes for length
const metadataExtension = TYPE_SIZE + LENGTH_SIZE;
// Size of metadata
const metadataLen = pack(metaData).length;

// Size of Mint Account with extension
const mintLen = getMintLen([ExtensionType.MetadataPointer]);

// Minimum lamports required for Mint Account
const lamports = await connection.getMinimumBalanceForRentExemption(
    mintLen + metadataExtension + metadataLen
);

// Instruction to invoke System Program to create new account
const createAccountInstruction = SystemProgram.createAccount({
    fromPubkey: payer.publicKey, // Account that will transfer lamports to created account
    newAccountPubkey: mint, // Address of the account to create
    space: mintLen, // Amount of bytes to allocate to the created account
    lamports, // Amount of lamports transferred to created account
    programId: TOKEN_2022_PROGRAM_ID, // Program assigned as owner of created account
});

// Instruction to initialize the MetadataPointer Extension
const initializeMetadataPointerInstruction =
    createInitializeMetadataPointerInstruction(
        mint, // Mint Account address
        updateAuthority, // Authority that can set the metadata address
        mint, // Account address that holds the metadata
        TOKEN_2022_PROGRAM_ID
    );

// Instruction to initialize Mint Account data
const initializeMintInstruction = createInitializeMintInstruction(
    mint, // Mint Account Address
    decimals, // Decimals of Mint
    mintAuthority, // Designated Mint Authority
    null, // Optional Freeze Authority
    TOKEN_2022_PROGRAM_ID // Token Extension Program ID
);

// Instruction to initialize Metadata Account data
const initializeMetadataInstruction = createInitializeInstruction({
    programId: TOKEN_2022_PROGRAM_ID, // Token Extension Program as Metadata Program
    metadata: mint, // Account address that holds the metadata
    updateAuthority: updateAuthority, // Authority that can update the metadata
    mint: mint, // Mint Account address
    mintAuthority: mintAuthority, // Designated Mint Authority
    name: metaData.name,
    symbol: metaData.symbol,
    uri: metaData.uri,
});

// Instruction to update metadata, adding custom field
const updateFieldInstruction = createUpdateFieldInstruction({
    programId: TOKEN_2022_PROGRAM_ID, // Token Extension Program as Metadata Program
    metadata: mint, // Account address that holds the metadata
    updateAuthority: updateAuthority, // Authority that can update the metadata
    field: metaData.additionalMetadata[0][0], // key
    value: metaData.additionalMetadata[0][1], // value
});

// Add instructions to new transaction
/*
transaction = new Transaction().add(
    createAccountInstruction, // 对应  #1 - System Program: createAccount
    initializeMetadataPointerInstruction, // 对应  #2 - Token 2022 Program: initializeMetadataPointer
    initializeMintInstruction, // 对应  #3 - Token 2022 Program: initializeMint
    initializeMetadataInstruction, // 对应  #4 - Token 2022 Program: initializeTokenMetadata
    updateFieldInstruction // 对应  #5 - Token 2022 Program: updateTokenMetadataField
);
*/

const MINT_ACCOUNT_SIZE = 500; // Token-2022 mint account 大小
// 现在应调整所需的是
const transfer = SystemProgram.transfer({
    fromPubkey: payer.publicKey,
    toPubkey: mint,
    lamports: await connection.getMinimumBalanceForRentExemption(MINT_ACCOUNT_SIZE),
})

// 设置最大 compute units，例如 400_000
const computeBudgetIx = ComputeBudgetProgram.setComputeUnitLimit({
    units: 400_000,
});

// 这条命令模拟的是
// spl-token initialize-metadata minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG 'Example' 'EXMPL'
// https://raw.githubusercontent.com/sunzhenvip/developer-bootcamp-2024/refs/heads/master/project-5-tokens/sample-token-metadata.json
transaction = new Transaction().add(
    transfer, // 对应  #1 - System Program: transfer
    initializeMetadataInstruction, // #2 - Token 2022 Program: initializeTokenMetadata
    computeBudgetIx, // #3 - Compute Budget: SetComputeUnitLimit
);

// Send transaction
transactionSignature = await sendAndConfirmTransaction(
    connection,
    transaction,
    [payer, mintKeypair] // Signers
);

console.log(
    "\nCreate Mint Account:",
    `https://solana.fm/tx/${transactionSignature}?cluster=devnet-solana`
);



// Retrieve mint information
const mintInfo = await getMint(
    connection,
    mint,
    "confirmed",
    TOKEN_2022_PROGRAM_ID
);

// Retrieve and log the metadata pointer state
const metadataPointer = getMetadataPointerState(mintInfo);
console.log("\nMetadata Pointer:", JSON.stringify(metadataPointer, null, 2));

// Retrieve and log the metadata state
const metadata = await getTokenMetadata(
    connection,
    mint // Mint Account address
);
console.log("\nMetadata:", JSON.stringify(metadata, null, 2));



/*
// Instruction to remove a key from the metadata
const removeKeyInstruction = createRemoveKeyInstruction({
    programId: TOKEN_2022_PROGRAM_ID, // Token Extension Program as Metadata Program
    metadata: mint, // Address of the metadata
    updateAuthority: updateAuthority, // Authority that can update the metadata
    key: metaData.additionalMetadata[0][0], // Key to remove from the metadata
    idempotent: true, // If the idempotent flag is set to true, then the instruction will not error if the key does not exist
});

// Add instruction to new transaction
transaction = new Transaction().add(removeKeyInstruction);

// Send transaction
transactionSignature = await sendAndConfirmTransaction(
    connection,
    transaction,
    [payer]
);

console.log(
    "\nRemove Additional Metadata Field:",
    `https://solana.fm/tx/${transactionSignature}?cluster=devnet-solana`
);
*/

// Retrieve and log the metadata state
const updatedMetadata = await getTokenMetadata(
    connection,
    mint // Mint Account address
);
console.log("\nUpdated Metadata:", JSON.stringify(updatedMetadata, null, 2));

console.log(
    "\nMint Account:",
    `https://solana.fm/address/${mint}?cluster=devnet-solana`
);

// https://solana.com/zh/developers/guides/token-extensions/metadata-pointer#mint-setup