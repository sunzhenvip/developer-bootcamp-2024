const fs = require("fs");
const anchor = require("@coral-xyz/anchor");
const switchboardIdl = require("../tests/switchboard-idl.json");

// 重要：本脚本只修补 solana account 下载到磁盘的本地账户快照，目的是让
// 2024 年的教学项目继续能在 solana-test-validator 上运行。它不会、也不能
// 修改主网上的 Queue 或 Oracle，不能把这种处理用于生产环境。

// 以下偏移来自当前 tests/switchboard-idl.json 中 OracleAccountData 的字段
// 布局。Anchor 账户开头有 8 字节 discriminator，所以字段位置都从 8 开始。
// 这里直接修改原始 buffer，是为了保留 CLI JSON 的其他内容和账户原始长度。
const ORACLE_DISCRIMINATOR_SIZE = 8;
const ENCLAVE_SIGNER_OFFSET = ORACLE_DISCRIMINATOR_SIZE;
const VERIFICATION_STATUS_OFFSET = ORACLE_DISCRIMINATOR_SIZE + 64;
const VERIFICATION_TIMESTAMP_OFFSET = ORACLE_DISCRIMINATOR_SIZE + 72;
const VALID_UNTIL_OFFSET = ORACLE_DISCRIMINATOR_SIZE + 80;
const SECP256K1_SIGNER_OFFSET = ORACLE_DISCRIMINATOR_SIZE + 184;
const STAGING_ED25519_SIGNER_OFFSET = ORACLE_DISCRIMINATOR_SIZE + 2408;
const STAGING_SECP256K1_SIGNER_OFFSET = ORACLE_DISCRIMINATOR_SIZE + 2440;
const QUEUE_ORACLE_KEYS_OFFSET = 8 + 32 + 32 * 32;
const QUEUE_ORACLE_KEYS_LEN_OFFSET = 5204;
const LOCAL_VERIFICATION_LIFETIME_SECONDS = 24 * 60 * 60;

const coder = new anchor.BorshAccountsCoder(switchboardIdl);

// 读取 setup-local.sh 生成的清单，并同时保留：
// - snapshotText：原始 JSON 文本，写回时避免大整数精度损失；
// - data：可以修改的链上账户二进制；
// - decoded：IDL 解码后的字段，方便判断 signer、有效期和 gateway 地址。
const manifest = fs
  .readFileSync("oracle-accounts.txt", "utf8")
  .trim()
  .split("\n")
  .map((line) => {
    const [address, file] = line.trim().split(/\s+/);
    const snapshotText = fs.readFileSync(file, "utf8");
    const snapshot = JSON.parse(snapshotText);
    const data = Buffer.from(snapshot.account.data[0], "base64");
    const decoded = coder.decode("OracleAccountData", data);
    const gatewayUrl = Buffer.from(decoded.gateway_uri)
      .toString("utf8")
      .replace(/\0+$/, "");

    return {
      address,
      file,
      snapshot,
      snapshotText,
      originalBase64: snapshot.account.data[0],
      data,
      decoded,
      gatewayUrl,
    };
  });

// SDK 的 commitIx 最终需要从 Oracle gateway 获取签名，因此账户存在还不够，
// gateway 也必须在线。5 秒超时可避免失效的旧网关拖住整个初始化过程。
async function gatewayIsOnline(gatewayUrl) {
  try {
    const response = await fetch(`${gatewayUrl}/gateway/api/v1/test`, {
      signal: AbortSignal.timeout(5_000),
    });
    return response.ok && (await response.text()).length > 0;
  } catch {
    return false;
  }
}

async function main() {
  // 并行探测全部 gateway，优先找在线且已有有效 active signer 的 Oracle；
  // 如果 active signer 已过期/清空，但存在 staging signer，也可以用于本地快照。
  const availability = await Promise.all(
    manifest.map(({ gatewayUrl }) => gatewayIsOnline(gatewayUrl))
  );
  const now = Math.floor(Date.now() / 1000);
  const selectedIndex = manifest.findIndex((oracle, index) => {
    if (!availability[index]) return false;

    const status = oracle.decoded.enclave.verification_status;
    const validUntil = Number(oracle.decoded.enclave.valid_until.toString());
    const hasActiveSigner = !oracle.decoded.enclave.enclave_signer.equals(
      anchor.web3.PublicKey.default
    );
    const hasStagingSigner =
      !oracle.decoded.enclave.staging_ed25519_signer.equals(
        anchor.web3.PublicKey.default
      );

    return (
      // verification_status === 4 表示已验证；额外要求至少还有 1 小时有效期，
      // 避免刚准备完本地环境 signer 就过期。
      (status === 4 && validUntil > now + 3_600 && hasActiveSigner) ||
      hasStagingSigner
    );
  });

  if (selectedIndex === -1) {
    throw new Error(
      "No online Switchboard oracle gateway with a usable signer"
    );
  }

  manifest.forEach((oracle, index) => {
    // 本地只保留一个 status=4 的 Oracle。旧版 @switchboard-xyz/on-demand
    // 会从 Queue 中随机选择 Oracle；若仍保留已经离线的旧成员，commitIx
    // 可能随机命中坏网关，导致同一测试有时成功、有时失败。
    oracle.data[VERIFICATION_STATUS_OFFSET] = index === selectedIndex ? 4 : 0;

    if (index === selectedIndex) {
      const stagingSigner = oracle.data.subarray(
        STAGING_ED25519_SIGNER_OFFSET,
        STAGING_ED25519_SIGNER_OFFSET + 32
      );
      const stagingSecp256k1Signer = oracle.data.subarray(
        STAGING_SECP256K1_SIGNER_OFFSET,
        STAGING_SECP256K1_SIGNER_OFFSET + 64
      );

      if (
        oracle.decoded.enclave.enclave_signer.equals(
          anchor.web3.PublicKey.default
        )
      ) {
        // 当前旧 Queue 中 active signer 已被清空，但网关账户里仍发布了新的
        // staging signer。仅在 active signer 为空时，把 staging signer 复制到
        // active 字段，让网关返回的签名能通过本地 Switchboard 程序校验。
        stagingSigner.copy(oracle.data, ENCLAVE_SIGNER_OFFSET);
        stagingSecp256k1Signer.copy(oracle.data, SECP256K1_SIGNER_OFFSET);
      }

      // 给选中的本地 Oracle 设置“现在验证、24 小时后过期”。这只是测试夹具
      // 的有效期，重新运行 setup-local.sh 时会从主网重新下载并再次生成。
      oracle.data.writeBigInt64LE(BigInt(now), VERIFICATION_TIMESTAMP_OFFSET);
      oracle.data.writeBigInt64LE(
        BigInt(now + LOCAL_VERIFICATION_LIFETIME_SECONDS),
        VALID_UNTIL_OFFSET
      );
    }

    // 不对整个 snapshot 执行 JSON.stringify：其中 rentEpoch 等 u64 数字可能
    // 大于 Number.MAX_SAFE_INTEGER，JS 解析后再序列化会发生精度损失。
    // 因此只在原始 JSON 文本中替换 base64 账户数据。
    fs.writeFileSync(
      oracle.file,
      oracle.snapshotText.replace(
        oracle.originalBase64,
        oracle.data.toString("base64")
      )
    );
  });

  const selected = manifest[selectedIndex];
  const queueSnapshotText = fs.readFileSync("randomness_queue.json", "utf8");
  const queueSnapshot = JSON.parse(queueSnapshotText);
  const originalQueueBase64 = queueSnapshot.account.data[0];
  const queueData = Buffer.from(originalQueueBase64, "base64");

  // 把“本地 Queue”收窄为刚选中的一个 Oracle。旧版 SDK 会在筛选有效状态前
  // 探测列表中的所有 gateway；即使测试已成功，离线连接仍可能让 Mocha 长时间
  // 不退出。这里只改本地 randomness_queue.json，不影响主网 Queue。
  new anchor.web3.PublicKey(selected.address)
    .toBuffer()
    .copy(queueData, QUEUE_ORACLE_KEYS_OFFSET);
  queueData.writeUInt32LE(1, QUEUE_ORACLE_KEYS_LEN_OFFSET);
  fs.writeFileSync(
    "randomness_queue.json",
    queueSnapshotText.replace(originalQueueBase64, queueData.toString("base64"))
  );

  // 记录本次选择，排查日志时可以快速确认请求实际发往哪个 Oracle。
  fs.writeFileSync("selected-oracle.txt", `${selected.address}\n`);
  console.log(`Selected local randomness oracle: ${selected.address}`);
  console.log(`Gateway: ${selected.gatewayUrl}`);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
