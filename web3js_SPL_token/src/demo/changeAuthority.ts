// 创建一个"更改 Token 账户所有者"的指令（Set Authority）
// 这个指令会将 Token 账户的所有权（Owner）转移给新账户
// 关键特性：
// - 转移后，新所有者可以操作该账户中的代币
// - 旧所有者失去对账户的控制权
// - 代币仍然在账户中，只是控制权转移
// - 只有当前所有者才能执行此操作
// 用途：
// - 出售账户：将 Token 账户出售给他人
// - 转移控制权：将账户管理权移交给他人
const changeTokenAuthorityInstruction = createSetAuthorityInstruction(
    tokenAccount,         // Account（Token 账户）：要更改所有者的 Token 账户地址
    keypair.publicKey,    // Current Authority：当前的所有者（必须是当前 Owner 才能转移）
    AuthorityType.AccountOwner,  // Authority Type：权限类型，这里指定为"账户所有者"
    newAuthority.publicKey // New Authority：新的所有者地址（将获得该账户的控制权）
);

// 创建一个"更改 Mint 的 Freeze Authority"的指令（Set Authority）
// 这个指令会修改 Mint 的冻结权限（Freeze Authority）
// 关键特性：
// - 新的 Freeze Authority 可以冻结/解冻该代币的任何 Token 账户
// - 可以将 Freeze Authority 设置为 null（放弃冻结权限，实现去中心化）
// - ⚠️ 设置为 null 后无法恢复（不可逆操作）
// - 只有当前的 Freeze Authority 才能执行此操作
// 用途：
// - 转移冻结权：将冻结权限转让给他人（如合规部门）
// - 放弃冻结权：设置为 null，实现完全去中心化
// - 合规管理：将冻结权转移给专门的合规团队
const changeMintFreezeAuthorityInstruction = createSetAuthorityInstruction(
    mint,                 // Account（Mint 账户）：Mint 地址（要修改哪个 Mint 的权限）
    keypair.publicKey,    // Current Authority：当前的 Freeze Authority（必须是当前权限持有者）
    AuthorityType.FreezeAccount,  // Authority Type：权限类型，这里指定为"冻结账户权限"
    newAuthority.publicKey // New Authority：新的 Freeze Authority 地址（将获得冻结权限）
                              // 设置为 null 表示放弃冻结权限（任何人都无法冻结账户）
);

// 创建一个"更改 Mint 的 Mint Authority"的指令（Set Authority）
// 这个指令会修改 Mint 的铸造权限（Mint Authority）
// 关键特性：
// - 新的 Mint Authority 可以铸造新代币
// - 可以将 Mint Authority 设置为 null（固定供应量，无法再铸造）
// - ⚠️ 设置为 null 后无法恢复（不可逆操作）
// - 只有当前的 Mint Authority 才能执行此操作
// 用途：
// - 转移铸币权：将铸造权转让给他人
// - 固定供应量：设置为 null，实现通缩或固定供应模型
// - 多签控制：转移给多签钱包，实现集体决策
const changeMintAuthorityInstruction = createSetAuthorityInstruction(
    mint,                 // Account（Mint 账户）：Mint 地址（要修改哪个 Mint 的权限）
    keypair.publicKey,    // Current Authority：当前的 Mint Authority（必须是当前权限持有者）
    AuthorityType.MintTokens,  // Authority Type：权限类型，这里指定为"铸造代币权限"
    newAuthority.publicKey // New Authority：新的 Mint Authority 地址（将获得铸造权限）
                            // 设置为 null 表示固定供应量（无法再铸造新代币）
);

// 创建一个新的交易，并将三条指令添加到交易中
// 交易按顺序执行：转移 Token 账户所有权、转移 Freeze Authority、转移 Mint Authority
const transaction = new Transaction().add(
    changeTokenAuthorityInstruction,        // 第 1 步：转移 Token 账户的所有权
    changeMintFreezeAuthorityInstruction,   // 第 2 步：转移 Freeze Authority
    changeMintAuthorityInstruction          // 第 3 步：转移 Mint Authority
);

// 发送交易到 Solana 网络并等待确认
// 需要提供签名者：keypair（必须是当前权限的持有者）
const signature = await sendAndConfirmTransaction(connection, transaction, [keypair]);

// 在控制台打印交易签名，并生成 Solana Explorer 的链接
console.log(`Token and Mint authority changed! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);