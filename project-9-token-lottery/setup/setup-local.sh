#!/usr/bin/env bash

set -euo pipefail

# 可以在命令行临时覆盖 RPC，例如：
# MAINNET_RPC_URL="https://..." ./setup-local.sh
# 默认值沿用项目里的 Helius key；此脚本只读取主网账户，不会向主网发送交易。
MAINNET_RPC_URL="${MAINNET_RPC_URL:-https://mainnet.helius-rpc.com/?api-key=c5730fdb-3471-42ff-92ad-97256fa83871}"
QUEUE_ADDRESS="A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w"

# Switchboard Queue 的成员会随时间轮换。原课程脚本写死了 2024 年的 8 个
# Oracle 地址；现在 Queue 里已经是另一批 Oracle，因此先下载最新 Queue，
# 再由 list-active-oracles.js 从 Queue 数据中提取当前真正有效的成员列表。
solana account --url "$MAINNET_RPC_URL" --output json-compact \
  --output-file randomness_queue.json "$QUEUE_ADDRESS"
node ./list-active-oracles.js randomness_queue.json > oracle-accounts.txt

# oracle-accounts.txt 每行格式为：<Oracle 公钥> <本地快照文件名>。
# 循环下载每个 Oracle 的主网账户快照，稍后交给本地 validator 加载。
while read -r oracle_address oracle_file; do
  solana account --url "$MAINNET_RPC_URL" --output json-compact \
    --output-file "$oracle_file" "$oracle_address"
done < oracle-accounts.txt

# 这是 2024 年课程使用的旧版 Queue。虽然账户仍在主网，但其中 Oracle 的
# attestation/active signer 可能已经过期。这里仅修改刚下载到本地的 JSON 快照：
# 选择一个当前网关可访问的 Oracle，并把它的 staging signer 临时提升为本地
# 测试使用的 active signer。不会修改任何主网账户。
node ./prepare-local-oracle.js

# 下载 Switchboard 配置账户，以及本地 validator 要加载的三个程序。
# --account 加载普通账户快照；--bpf-program 则在 start-validator.sh 中加载程序。
solana account --url "$MAINNET_RPC_URL" --output json-compact \
  --output-file sb_randomness_config.json 7Gs9n5FQMeC9XcEhg281bRZ6VHRrCvqp5Yq1j78HkvNa
solana program dump --url "$MAINNET_RPC_URL" \
  SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv ondemand.so
solana program dump --url "$MAINNET_RPC_URL" \
  SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f switchboard.so
solana program dump --url "$MAINNET_RPC_URL" \
  metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s metadata.so
