// 创建一个"解冻 Token 账户"的指令（Thaw Account）
// 这个指令会解冻之前被冻结的 Token 账户，使其恢复转账功能
// 解冻后，该账户的代币可以正常转出
// 关键特性：
// - 解冻后，账户恢复正常转账功能（解除冻结限制）
// - 代币仍然在账户中，所有权不变
// - 可以再次冻结（使用 Freeze Account 指令）
// - 只有拥有 Freeze Authority 的账户才能执行此操作
// 用途：
// - 解除合规冻结：监管要求解除后恢复账户使用
// - 恢复账户权限：安全事件处理完成后解冻
// - 法律程序结束：纠纷解决后解冻账户
// - 取消惩罚：违规行为改正后解冻
const thawInstruction = createThawAccountInstruction(
    tokenAccount,         // Account（Token 账户）：要解冻的 Token 账户地址
    mint,                 // Mint：代币的 Mint 地址（指定要解冻的是哪种代币）
    keypair.publicKey     // Freeze Authority：拥有冻结权限的账户（必须与冻结时使用的账户一致）
);

// 创建一个新的交易，并将解冻指令添加到交易中
// 这个交易只需要一条指令：解冻 Token 账户
const transaction = new Transaction().add(thawInstruction);

// 发送交易到 Solana 网络并等待确认
// 只需要一个签名者：keypair（Freeze Authority）
const signature = await sendAndConfirmTransaction(connection, transaction, [keypair]);

// 在控制台打印交易签名，并生成 Solana Explorer 的链接
console.log(`Token account thawed! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);