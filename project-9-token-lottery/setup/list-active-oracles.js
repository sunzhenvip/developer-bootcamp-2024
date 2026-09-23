const fs = require("fs");
const anchor = require("@coral-xyz/anchor");
const switchboardIdl = require("../tests/switchboard-idl.json");

// solana account --output json-compact 把链上账户数据放在
// account.data[0] 中，并使用 base64 编码。这里用 Switchboard IDL 按 Borsh
// 格式解码，避免通过固定字节位置猜测 Queue 的成员。
const queueSnapshotPath = process.argv[2] ?? "randomness_queue.json";
const queueSnapshot = JSON.parse(fs.readFileSync(queueSnapshotPath, "utf8"));
const queueData = new anchor.BorshAccountsCoder(switchboardIdl).decode(
  "QueueAccountData",
  Buffer.from(queueSnapshot.account.data[0], "base64")
);

// oracle_keys 是固定容量数组（当前长度为 78），空余槽位也包含在数组中；
// oracle_keys_len 才是链上记录的实际成员数。原脚本的 8 个地址属于 2024 年
// 的旧成员，不能继续写死，所以只截取目前处于 Queue 中的有效部分。
const oracleKeys = queueData.oracle_keys.slice(0, queueData.oracle_keys_len);

if (oracleKeys.length === 0) {
  throw new Error("The Switchboard queue has no active oracle accounts");
}

// 输出会被 setup-local.sh 重定向到 oracle-accounts.txt。
// 第二列只是可预测的本地文件名，start-validator.sh 会读取同一份清单。
oracleKeys.forEach((oracleKey, index) => {
  console.log(`${oracleKey.toBase58()} oracle${index}.json`);
});
