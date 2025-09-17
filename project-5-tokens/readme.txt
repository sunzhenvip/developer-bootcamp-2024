
TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb(表示 token2022)
solana address
sunkq6xnqcHMqrrBX7wPTHS9sVCZ9SNTentvHyFSnxh
此网站领取测试币开发网|测试网 https://faucet.solana.com/

1、yarn add @metaplex-foundation/mpl-token-metadata@3.2.1
2、solana config set -u devnet(设置开发环境) 用完切换回本地环境 solana config set -u localhost
3、solana-keygen grind --starts-with min:1

Searching with 12 threads for:
        1 pubkey that starts with 'min' and ends with ''
Searched 1000000 keypairs in 2s. 0 matches found.
Searched 2000000 keypairs in 5s. 0 matches found.
Searched 3000000 keypairs in 8s. 0 matches found.
Wrote keypair to minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG.json

4、spl-token create-token --program-id TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb --enable-metadata minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG.json

Creating token minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG under program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
To initialize metadata inside the mint, please run `spl-token initialize-metadata minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG <YOUR_TOKEN_NAME> <YOUR_TOKEN_SYMBOL> <YOUR_TOKEN_URI>`, and sign with the mint authority.

Address:  minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG
Decimals:  9

Signature: 5gFKqnSDLXxKi5sNFLBr8HVHSyxcRA5ykfWCVPq8JMKK9gsFhBmND7WTENpVYx6kLUryZ5ut8XWGFUd8uZhDbz5B

5、spl-token initialize-metadata minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG 'Example' 'EXMPL' https://raw.githubusercontent.com/sunzhenvip/developer-bootcamp-2024/refs/heads/master/project-5-tokens/sample-token-metadata.json

Signature: 22fkWpgAyFg9hYP6uoLS87xPmFFmgBMx29mTsoUoiD8ToeEA5wXSCHnv8WjEfxzfZnohaMiPC87A5rfjvFYm1B7Z


