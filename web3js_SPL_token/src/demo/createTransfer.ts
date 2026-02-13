// 生成一个新的随机密钥对，作为接收代币的目标账户（Recipient）
// 这个账户将接收我们转账的代币
const destination = Keypair.generate();

// 计算目标账户的关联代币账户（ATA）地址
// ATA 地址由 Mint + Owner 唯一确定，通过算法派生
const destinationTokenAccount = await getAssociatedTokenAddress(
    mint.publicKey,         // Mint 地址（哪种代币）
    destination.publicKey,  // 接收者的地址（谁将接收代币）
);

// 创建一个"创建关联代币账户"的指令
// 使用幂等版本：如果 ATA 已存在则跳过，不会报错
// 这样可以确保目标账户存在，否则无法接收代币
const createAtaInstruction = createAssociatedTokenAccountIdempotentInstruction(
    keypair.publicKey,          // Payer：谁支付创建 ATA 的费用
    destinationTokenAccount,    // ATA 的地址（计算出来的派生地址）
    destination.publicKey,      // Owner：谁拥有这个 ATA（接收代币的人）
    mint.publicKey,             // Mint：这个 ATA 持有的是哪种代币
);

// 创建一个"转账代币"的指令（Transfer）
// 这个指令会将代币从源账户转移到目标账户
// 注意：只有源账户的所有者（Owner）才能执行此操作
const transferInstruction = createTransferInstruction(
    sourceTokenAccount,      // Source（源 Token 账户）：从哪个账户转出代币
    destinationTokenAccount, // Destination（目标 Token 账户）：代币转入到这个账户
    keypair.publicKey,       // Owner of Source：源账户的所有者（必须有权限操作源账户）
    1e6,                     // Amount：要转账多少代币（使用最小单位）
                             // 1e6 = 1 × 10^6 = 1,000,000 最小单位
                             // 如果代币有 6 位小数，这表示 1 个完整代币
);

// 创建一个新的交易，并将两条指令添加到交易中
// 交易按顺序执行：先创建目标 ATA，再转账代币
const transaction = new Transaction().add(
    createAtaInstruction,    // 第 1 步：确保目标 ATA 存在（如果不存在则创建）
    transferInstruction,     // 第 2 步：从源账户转账代币到目标账户
);

// 发送交易到 Solana 网络并等待确认
// 只需要一个签名者：keypair（既是 payer 又是源账户的 owner）
const signature = await sendAndConfirmTransaction(connection, transaction, [keypair]);

// 在控制台打印交易签名，并生成 Solana Explorer 的链接
console.log(`Token accounts created and tokens transferred! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);