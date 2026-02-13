// 创建一个"销毁代币"的指令（Burn）
// 这个指令会永久销毁指定 Token 账户中的代币，减少代币的总供应量
// 注意：只有 Token 账户的所有者（Owner）才能执行此操作
// 销毁是不可逆的，一旦执行，代币将永远消失，无法恢复
const burnInstruction = createBurnInstruction(
    tokenAccount,         // Account（Token 账户）：从哪个账户销毁代币（源账户）
    mint,                 // Mint：代币的 Mint 地址（指定要销毁的是哪种代币）
    keypair.publicKey,    // Owner：Token 账户的所有者（必须有权限操作该账户）
    1e6,                  // Amount：要销毁多少代币（使用最小单位）
                          // 1e6 = 1 × 10^6 = 1,000,000 最小单位
                          // 如果代币有 6 位小数，这表示 1 个完整代币
);

// 创建一个新的交易，并将销毁指令添加到交易中
// 这个交易只需要一条指令：销毁代币
const transaction = new Transaction().add(burnInstruction);

// 发送交易到 Solana 网络并等待确认
// 只需要一个签名者：keypair（Token 账户的 owner）
const signature = await sendAndConfirmTransaction(connection, transaction, [keypair]);

// 在控制台打印交易签名，并生成 Solana Explorer 的链接
console.log(`Tokens Burned! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);