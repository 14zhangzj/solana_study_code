// 创建一个"撤销授权委托"的指令（Revoke）
// 这个指令会撤销之前通过 Approve 授予 Delegate 的权限
// 撤销后，Delegate 将无法再使用你的代币进行转账
// 关键特性：
// - 立即生效，撤销后 Delegate 的权限立即失效
// - 不会影响已经转账的代币（只影响未来的转账）
// - 可以重新授权（使用 Approve 指令）
// - 只能由 Token 账户的所有者（Owner）执行
// 用途：
// - 取消 DeFi 协议的授权（使用完协议后）
// - 紧急情况下停止 Delegate 的权限
// - 定期安全管理（撤销不再需要的授权）
const revokeInstruction = createRevokeInstruction(
    tokenAccount,         // Account（Token 账户）：要撤销授权的 Token 账户地址
    keypair.publicKey     // Owner：Token 账户的所有者（只有所有者才能撤销授权）
);

// 创建一个新的交易，并将撤销授权指令添加到交易中
// 这个交易只需要一条指令：撤销授权
const transaction = new Transaction().add(revokeInstruction);

// 发送交易到 Solana 网络并等待确认
// 只需要一个签名者：keypair（Token 账户的 owner）
const signature = await sendAndConfirmTransaction(connection, transaction, [keypair]);

// 在控制台打印交易签名，并生成 Solana Explorer 的链接
console.log(`Token account delegate revoked! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);