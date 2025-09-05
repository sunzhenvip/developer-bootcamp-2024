solana account -u m --output json-compact --output-file oracle9.json H6sA5hiCJkUHM6xGabpg5qGQXNqUgAT7S6FDapqfVEqx
solana account -u m --output json-compact --output-file oracle8.json B6myYsSNLmdbQxwXVhBQZsjbSVLRtoRhYF3hMFeRyftF
solana account -u m --output json-compact --output-file oracle7.json CdAPHuaduAH4NmxC8AvCqWuiQ1LduTrawnFBJKBdjPy4
solana account -u m --output json-compact --output-file oracle6.json 31Uys8oYqNAiRUKR9i24qLaG5ninMFuXckpkfV3FaPDp
solana account -u m --output json-compact --output-file oracle5.json 5LD5BjwuNN397r4xDubXhhdxX3SSx9wS4ub33J64pjfW
solana account -u m --output json-compact --output-file oracle4.json Bcup5msLa9wK6FxnSCZDQ6CGBuoAD6Wm78Epgi3KwGzV
solana account -u m --output json-compact --output-file oracle3.json 5wCwgqgPtFB9jwjZxLVkM717SGaZKmXXpvXYsyLehu69
solana account -u m --output json-compact --output-file oracle2.json 5eVyN3Wx88y3d19kvYC9wBhdaZAwNdmKeA3LiXKEm9hH
solana account -u m --output json-compact --output-file oracle1.json 8ev3ovH86XmD45JU6YhPy6B3ZVZonixLMVGEcw1B6gwC
solana account -u m --output json-compact --output-file oracle0.json 3Nv1DJdf7163FcB5dFEQGKbw6dUK4HqtwuUcyUf3DWni
solana account -u m --output json-compact --output-file randomness_queue.json A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w
solana account -u m --output json-compact --output-file sb_randomness_config.json 7Gs9n5FQMeC9XcEhg281bRZ6VHRrCvqp5Yq1j78HkvNa
solana program dump -u m SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv ondemand.so
solana program dump -u m SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f switchboard.so
solana program dump -u m metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s metadata.so



solana-test-validator -r \
    --account H6sA5hiCJkUHM6xGabpg5qGQXNqUgAT7S6FDapqfVEqx oracle9.json \
    --account B6myYsSNLmdbQxwXVhBQZsjbSVLRtoRhYF3hMFeRyftF oracle8.json \
    --account CdAPHuaduAH4NmxC8AvCqWuiQ1LduTrawnFBJKBdjPy4 oracle7.json \
    --account 31Uys8oYqNAiRUKR9i24qLaG5ninMFuXckpkfV3FaPDp oracle6.json \
    --account 5LD5BjwuNN397r4xDubXhhdxX3SSx9wS4ub33J64pjfW oracle5.json \
    --account Bcup5msLa9wK6FxnSCZDQ6CGBuoAD6Wm78Epgi3KwGzV oracle4.json \
    --account 5wCwgqgPtFB9jwjZxLVkM717SGaZKmXXpvXYsyLehu69 oracle3.json \
    --account 5eVyN3Wx88y3d19kvYC9wBhdaZAwNdmKeA3LiXKEm9hH oracle2.json \
    --account 8ev3ovH86XmD45JU6YhPy6B3ZVZonixLMVGEcw1B6gwC oracle1.json \
    --account 3Nv1DJdf7163FcB5dFEQGKbw6dUK4HqtwuUcyUf3DWni oracle0.json \
    --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s metadata.so \
    --bpf-program SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f switchboard.so\
    --account A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w randomness_queue.json \
    --bpf-program SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv ondemand.so \
    --account 7Gs9n5FQMeC9XcEhg281bRZ6VHRrCvqp5Yq1j78HkvNa sb_randomness_config.json



solana program dump -u m SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv ondemand.so
solana program dump -u m SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f switchboard.so
solana program dump -u m metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s metadata.so



solana program dump -u d Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2 ondemand.so
solana program dump -u m SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f switchboard.so
solana program dump -u m metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s metadata.so


solana-keygen new --outfile metadata.json
solana-keygen new --outfile switchboard.json
solana-keygen new --outfile ondemand.json
solana-keygen new --outfile local-queue.json


solana-keygen pubkey metadata.json
solana-keygen pubkey switchboard.json
solana-keygen pubkey ondemand.json
solana-keygen pubkey local-queue.json
solana-keygen pubkey /home/sz/.config/solana/id.json


solana-test-validator --reset \
--bpf-program $(solana-keygen pubkey metadata.json) metadata.so \
--bpf-program $(solana-keygen pubkey switchboard.json) switchboard.so \
--bpf-program $(solana-keygen pubkey ondemand.json) ondemand.so





solana-test-validator --reset \
--bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s metadata.so \
--bpf-program SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f switchboard.so \
--bpf-program Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2 ondemand.so



sb solana on-demand queue init \
  --keypair local-queue.json \
  --rpcUrl http://127.0.0.1:8899 \
  --programId Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2





sb solana on-demand queue init \
  --keypair local-queue.json \
  --rpcUrl http://127.0.0.1:8899 \
  --programId $(solana-keygen pubkey ondemand.json)




npm install -g @switchboard-xyz/cli^2


npm install -g @switchboard-xyz/cli@2.1.27

sb solana on-demand queue init --keypair local-keypair.json --rpcUrl http://127.0.0.1:8899


sb config set solana localnet keypair /data/network/rust/web3/developer-bootcamp-2024/project-9-token-lottery/setup/local-keypair.json


sb config set solana localnet default-account "/data/network/rust/web3/developer-bootcamp-2024/project-9-token-lottery/setup/local-keypair.json"

# https://docs.switchboard.xyz/tooling-and-resources/technical-resources-and-documentation/solana-accounts 查看主网队列公钥
# 获取主网的队列命令
# npm install -g @switchboard-xyz/cli 安装 sb 命令行
sb solana on-demand queue print A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w --cluster mainnet-beta