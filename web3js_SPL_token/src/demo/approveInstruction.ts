// 生成一个新的随机密钥对，作为代币账户的所有者（Owner）
// 这个账户将拥有代币，并授权其他人使用其中的代币
const keypair = Keypair.generate();

// 计算所有者的关联代币账户（ATA）地址
// ATA 地址由 Mint + Owner 唯一确定，通过算法派生
const tokenAccount = await getAssociatedTokenAddress(
    mint.publicKey,         // Mint 地址（哪种代币）
    keypair.publicKey,      // Owner 的地址（谁拥有这个 ATA）
);

// 创建一个"创建关联代币账户"的指令
// 使用幂等版本：如果 ATA 已存在则跳过，不会报错
// 这样可以确保目标账户存在，否则无法进行授权操作
const createAtaInstruction = createAssociatedTokenAccountIdempotentInstruction(
    keypair.publicKey,      // Payer：谁支付创建 ATA 的费用
    tokenAccount,           // ATA 的地址（计算出来的派生地址）
    destination.publicKey,  // Owner：谁拥有这个 ATA
    mint.publicKey,         // Mint：这个 ATA 持有的是哪种代币
);

// 创建一个"授权委托"的指令（Approve / Delegate）
// 这个指令会授权指定的账户（Delegate）可以使用你 Token 账户中的代币
// 授权后，Delegate 可以代替你转账代币，最多转账授权的金额
// 关键特性：
// - 代币不会离开你的账户，只是授权他人使用
// - 可以设置授权金额上限（Delegate 最多能转走这么多）
// - 可以随时撤销授权（使用 Revoke 指令）
// 用途：
// - DeFi 协议：授权协议使用你的代币进行交易
// - 自动支付：授权服务商定期扣款
// - 第三方服务：授权他人管理你的代币
const approveInstruction = createApproveInstruction(
    tokenAccount,         // Account（Token 账户）：要授权的 Token 账户地址
    delegate.publicKey,   // Delegate：被授权的账户地址（这个账户可以使用代币）
    keypair.publicKey,    // Owner：Token 账户的所有者（只有所有者才能授权）
    1e6,                  // Amount：授权的金额上限（使用最小单位）
                          // 1e6 = 1 × 10^6 = 1,000,000 最小单位
                          // 如果代币有 6 位小数，这表示 Delegate 最多能转走 1 个代币
);

// 创建一个新的交易，并将两条指令添加到交易中
// 交易按顺序执行：先创建 ATA，再进行授权
const transaction = new Transaction().add(
    createAtaInstruction,    // 第 1 步：确保 ATA 存在（如果不存在则创建）
    approveInstruction,      // 第 2 步：授权 Delegate 使用代币
);

// 发送交易到 Solana 网络并等待确认
// 只需要一个签名者：keypair（Token 账户的 owner）
const signature = await sendAndConfirmTransaction(connection, transaction, [keypair]);

// 在控制台打印交易签名，并生成 Solana Explorer 的链接
console.log(`Token accounts created and delegated! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);