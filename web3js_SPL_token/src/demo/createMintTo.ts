// 生成一个新的随机密钥对，作为接收代币的目标账户（Recipient）
// 这个账户将接收我们铸造的代币
const destination = Keypair.generate();

// 计算目标账户的关联代币账户（ATA）地址
// ATA 地址由 Mint + Owner 唯一确定，通过算法派生
const tokenAccount = await getAssociatedTokenAddress(
    mint.publicKey,         // Mint 地址（要铸造哪种代币）
    destination.publicKey,  // 接收者的地址（谁将接收代币）
);

// 创建一个"创建关联代币账户"的指令
// 使用幂等版本：如果 ATA 已存在则跳过，不会报错
// 这样可以确保目标账户存在，否则无法接收代币
const createAtaInstruction = createAssociatedTokenAccountIdempotentInstruction(
    keypair.publicKey,      // Payer：谁支付创建 ATA 的费用
    tokenAccount,           // ATA 的地址（计算出来的派生地址）
    destination.publicKey,  // Owner：谁拥有这个 ATA（接收代币的人）
    mint.publicKey,         // Mint：这个 ATA 持有的是哪种代币
);

// 创建一个"铸造代币"的指令（MintTo）
// 这个指令会将新代币铸造到指定的 Token 账户中
// 注意：只有拥有 Mint Authority 的账户才能执行此操作
const mintToInstruction = createMintToInstruction(
    mint.publicKey,         // Mint 地址（从哪个 Mint 铸造代币）
    tokenAccount,           // Destination（目标 Token 账户）：代币将铸造到这个账户
    keypair.publicKey,      // Mint Authority（铸币权限）：必须有 Mint Authority 才能铸造
    1_000e6,                // Amount：要铸造多少代币（使用最小单位）
                           // 1_000e6 = 1_000 * 10^6 = 1,000,000,000 最小单位
                           // 如果代币有 6 位小数，这表示 1,000 个完整代币
);

// 创建一个新的交易，并将两条指令添加到交易中
// 交易按顺序执行：先创建 ATA，再铸造代币
const transaction = new Transaction().add(
    createAtaInstruction,    // 第 1 步：确保 ATA 存在（如果不存在则创建）
    mintToInstruction,       // 第 2 步：向 ATA 铸造代币
);

// 发送交易到 Solana 网络并等待确认
// 只需要一个签名者：keypair（既是 payer 又是 mint authority）
const signature = await sendAndConfirmTransaction(connection, transaction, [keypair]);

// 在控制台打印交易签名，并生成 Solana Explorer 的链接
console.log(`Token accounts created and tokens minted! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);