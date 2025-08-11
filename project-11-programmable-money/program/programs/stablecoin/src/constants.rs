use anchor_lang::prelude::*;

/**
    这个常量文件定义了稳定币协议的核心参数和配置值。
    PDA种子确保了账户地址的确定性和唯一性，而价格预言机常量保证了价格数据的准确性和时效性
    协议配置常量则直接影响系统的风险管理策略，决定了抵押率要求和清算机制的触发条件。
**/

// PDA种子常量 - 用于生成程序派生地址
pub const SEED_CONFIG_ACCOUNT: &[u8] = b"config"; // 配置账户PDA种子
pub const SEED_COLLATERAL_ACCOUNT: &[u8] = b"collateral"; // 抵押品账户PDA种子
pub const SEED_SOL_ACCOUNT: &[u8] = b"sol"; // SOL存储账户PDA种子
pub const SEED_MINT_ACCOUNT: &[u8] = b"mint"; // 稳定币铸造账户PDA种子

// Pyth价格预言机相关常量
#[constant]
pub const FEED_ID: &str = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
// SOL/USD价格预言机的Feed ID，用于获取SOL的实时价格数据
pub const MAXIMUM_AGE: u64 = 100; // allow pricefeed 100 sec old, to avoid stale price feed errors
                                  // 允许价格数据的最大过期时间（秒），防止使用过时的价格数据
pub const PRICE_FEED_DECIMAL_ADJUSTMENT: u128 = 10; // price feed returns 1e8, multiple by 10 to match lamports 10e9
                                                    // 价格数据小数位调整常量
                                                    // Pyth返回的价格精度是1e8，乘以10调整为lamports的1e9精度

// 协议配置默认值常量
// Constants for configuration values
pub const LIQUIDATION_THRESHOLD: u64 = 50; // 200% over-collateralized
                                           // 清算阈值：50%，意味着需要200%的超额抵押
                                           // 例如：存入200美元SOL才能铸造100美元稳定币

pub const LIQUIDATION_BONUS: u64 = 10; // 10% bonus lamports when liquidating
                                       // 清算奖励：10%，清算者执行清算时获得的额外lamport奖励

pub const MIN_HEALTH_FACTOR: u64 = 1;
// 最小健康因子：1，低于此值的仓位可被清算

pub const MINT_DECIMALS: u8 = 9;
// 稳定币的小数位数：9位，与SOL的lamports精度保持一致
