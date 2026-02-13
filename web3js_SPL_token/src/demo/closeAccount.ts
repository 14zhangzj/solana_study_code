// 创建一个"关闭 Token 账户"的指令（Close Account）
// 这个指令会永久关闭 Token 账户，并将账户中的租金余额（lamports）转移到指定地址
// 关闭账户的前提条件：
// 1. Token 账户的余额必须为 0（没有代币）
// 2. 只有 Token 账户的所有者（Owner）才能执行此操作
// 用途：
// - 清理不再使用的 Token 账户
// - 取回账户的租金（约 0.002 SOL）
// - 减少链上账户数量，节省存储空间
const closeAccountInstruction = createCloseAccountInstruction(
    tokenAccount,         // Account（Token 账户）：要关闭的 Token 账户地址
    keypair.publicKey,    // Destination：接收租金的地址（账户关闭后，租金余额会转到这个地址）
    keypair.publicKey     // Authority：Token 账户的所有者（必须有权限操作该账户）
);

// 创建一个新的交易，并将关闭账户指令添加到交易中
// 这个交易只需要一条指令：关闭 Token 账户
const transaction = new Transaction().add(closeAccountInstruction);

// 发送交易到 Solana 网络并等待确认
// 只需要一个签名者：keypair（Token 账户的 owner）
const signature = await sendAndConfirmTransaction(connection, transaction, [keypair]);

// 在控制台打印交易签名，并生成 Solana Explorer 的链接
console.log(`Token accounts closed! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);