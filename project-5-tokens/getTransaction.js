import {Connection, clusterApiUrl} from "@solana/web3.js";

async function main() {
    // 你可以换成 mainnet-beta / devnet / testnet
    const connection = new Connection(clusterApiUrl("mainnet-beta"), "confirmed");

    // 签名 1（正常返回的）
    const sig1 = "3v2tw13FiY4siERRuRtJr3hnnanq3mZWXLuyrhJWwRH25jcHUmNvLaTmUrvLueEjhEuP2wd5GwRYt1yiDfDiHzT9";
    // 签名 2（你说链上有但报错的）
    const sig2 = "zq4rAB13cowfyYqio9do2P4iHZzefGGYGF9JRRFwxi2aDmjChYMvVojbShELaKBK67wv3PzJAmYJMv34DE7NAVf";

    for (const sig of [sig1, sig2]) {
        try {
            console.log(`查询交易: ${sig}`);
            const tx = await connection.getTransaction(sig, {
                maxSupportedTransactionVersion: 0,
            });

            if (!tx) {
                console.log("❌ 没找到交易（可能被节点裁剪掉了，或者根本不是合法签名）");
            } else {
                console.log("✅ 找到交易，slot =", tx.slot);
            }
        } catch (err) {
            console.error("⚠️ 出错：", err);
        }
    }
}

main();

// npx esrun getTransaction.js