#!/usr/bin/env bash

set -euo pipefail

# Bash 数组最终会展开成多组：
# --account <Oracle 公钥> <对应 JSON 快照>
# 这样 Oracle 数量变化后不需要再手工修改本脚本。
oracle_accounts=()
while read -r oracle_address oracle_file; do
  oracle_accounts+=(--account "$oracle_address" "$oracle_file")
done < oracle-accounts.txt

if (( ${#oracle_accounts[@]} == 0 )); then
  echo "oracle-accounts.txt is empty; run setup-local.sh first" >&2
  exit 1
fi

# 请从 setup 目录运行本脚本，这样程序文件、账户快照以及 test-ledger 都会
# 留在同一目录，便于排查和后续删除。
#
# 加载内容：
#   1. 当前 Queue 的 Oracle 账户快照（上面的动态参数）；
#   2. Metaplex Token Metadata 和两个 Switchboard 程序；
#   3. Switchboard Queue 与 Randomness 配置账户。
#
# -r 表示每次启动都重建本地账本，避免上一次测试状态污染本次结果。
solana-test-validator \
  "${oracle_accounts[@]}" \
  --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s metadata.so \
  --bpf-program SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f switchboard.so \
  --account A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w randomness_queue.json \
  --bpf-program SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv ondemand.so \
  --account 7Gs9n5FQMeC9XcEhg281bRZ6VHRrCvqp5Yq1j78HkvNa sb_randomness_config.json \
  -r
