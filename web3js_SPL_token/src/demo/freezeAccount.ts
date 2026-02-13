// 生成一个新的随机密钥对，作为代币账户的所有者（Owner）
// 这个账户将拥有代币，其 Token 账户将被冻结
const keypair = Keypair.generate();

// 计算所有者的关联代币账户（ATA）地址
// ATA 地址由 Mint + Owner 唯一确定，通过算法派生
const tokenAccount = await getAssociatedTokenAddress(
    mint.publicKey,         // Mint 地址（哪种代币）
    keypair.publicKey,      // Owner 的地址（谁拥有这个 ATA）
);

// 创建一个"创建关联代币账户"的指令
// 使用幂等版本：如果 ATA 已存在则跳过，不会报错
// 这样可以确保目标账户存在，否则无法进行冻结操作
const createAtaInstruction = createAssociatedTokenAccountIdempotentInstruction(
    keypair.publicKey,      // Payer：谁支付创建 ATA 的费用
    tokenAccount,           // ATA 的地址（计算出来的派生地址）
    destination.publicKey,  // Owner：谁拥有这个 ATA
    mint.publicKey,         // Mint：这个 ATA 持有的是哪种代币
);

// 创建一个"冻结 Token 账户"的指令（Freeze Account）
// 这个指令会冻结指定的 Token 账户，使其无法进行转账操作
// 关键特性：
// - 冻结后，账户中的代币无法转出（转账操作会被拒绝）
// - 代币仍然在账户中，所有权不变
// - 可以解冻（使用 Thaw Account 指令）
// - 只有拥有 Freeze Authority 的账户才能执行此操作
// 用途：
// - 合规要求：监管要求的资产冻结
// - 安全保护：异常账户的临时冻结
// - 法律纠纷：法律程序中的资产冻结
// - 惩罚措施：违规行为的惩罚
const freezeInstruction = createFreezeAccountInstruction(
    tokenAccount,         // Account（Token 账户）：要冻结的 Token 账户地址
    mint,                 // Mint：代币的 Mint 地址（指定要冻结的是哪种代币）
    keypair.publicKey     // Freeze Authority：拥有冻结权限的账户（必须与 Mint 的 Freeze Authority 一致）
);

// 创建一个新的交易，并将两条指令添加到交易中
// 交易按顺序执行：先创建 ATA，再冻结账户
const transaction = new Transaction().add(
    createAtaInstruction,    // 第 1 步：确保 ATA 存在（如果不存在则创建）
    freezeInstruction,       // 第 2 步：冻结 Token 账户
);

// 发送交易到 Solana 网络并等待确认
// 只需要一个签名者：keypair（Freeze Authority）
const signature = await sendAndConfirmTransaction(connection, transaction, [keypair]);

// 在控制台打印交易签名，并生成 Solana Explorer 的链接
console.log(`Token accounts created and frozen! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);