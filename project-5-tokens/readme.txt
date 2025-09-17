
TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb(表示 token2022)
solana address
sunkq6xnqcHMqrrBX7wPTHS9sVCZ9SNTentvHyFSnxh
此网站领取测试币开发网|测试网 https://faucet.solana.com/

yarn add @metaplex-foundation/mpl-token-metadata@2.13.0

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

6、spl-token create-account minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG(创建PDA账户 有疑问 是 PDA ATA账户？)

Creating account Ed1S7TtPwoG7XgsAeb7QRZ79ykUdXnKf87vXqUpgvNgF

Signature: 4b3h3QXRrrhFFVL6u3YEwAGSbToztYoKBc95Xn94TCMrj82Q6hZXGmfwBJn8tQ4bU5YZ7tewt9zbqfyGSSoiZsqH

7、spl-token mint minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG 1000
Minting 1000 tokens
  Token: minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG
  Recipient: Ed1S7TtPwoG7XgsAeb7QRZ79ykUdXnKf87vXqUpgvNgF

Signature: 37cnxaDsEznTqw94KkKkXXUPnt689gz7BZ1r4srgoASHoByd9gu99bT6G7U3NvQpNDU9JrxtaEdkbMBy685YbbdT


8、spl-token accounts --owner sunkq6xnqcHMqrrBX7wPTHS9sVCZ9SNTentvHyFSnxh(用户钱包地址)
9、spl-token account-info  minYurXwc6ghWsZpabot8EnDSmG7K7QqnRjod9mDeEG(代币地址)


solana account tescD95ij1c27G7mDHXzN8wY3M6AuzN7nv6F5u813RV

4、spl-token create-token --program-id TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb --enable-metadata tescD95ij1c27G7mDHXzN8wY3M6AuzN7nv6F5u813RV.json

Creating token tescD95ij1c27G7mDHXzN8wY3M6AuzN7nv6F5u813RV under program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
To initialize metadata inside the mint, please run `spl-token initialize-metadata tescD95ij1c27G7mDHXzN8wY3M6AuzN7nv6F5u813RV <YOUR_TOKEN_NAME> <YOUR_TOKEN_SYMBOL> <YOUR_TOKEN_URI>`, and sign with the mint authority.

Address:  tescD95ij1c27G7mDHXzN8wY3M6AuzN7nv6F5u813RV
Decimals:  9

Signature: 5MoPvMNUBHsuAE7SZQwJLTxfy4TTn2J5JJP2gAu3P4P7i7DvjyjSSzwAquoqsQ1DuDusEZEKoYg4Jp9K55nxwyEx