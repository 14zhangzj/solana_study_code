// 从 @solana/web3.js 导入所需的 Solana 核心功能
import {
    Keypair,                      // 密钥对类，用于生成公钥和私钥
    sendAndConfirmTransaction,     // 发送交易并等待确认的函数
    SystemProgram,                 // 系统程序，用于创建账户等系统级操作
    Transaction,                   // 交易类，用于打包和发送指令
} from "@solana/web3.js";

// 从 @solana/spl-token 导入 SPL Token 相关的功能
import {
    createInitializeAccount3Instruction, // 创建初始化 Token 账户的指令
    ACCOUNT_SIZE,                         // Token 账户所需的空间大小（字节）
    getMinimumBalanceForRentExemptAccount, // 获取 Token 账户免租金所需的最低 lamports
    TOKEN_PROGRAM_ID,                     // Token Program 的程序 ID
} from "@solana/spl-token";

// 生成一个新的随机密钥对，这个将作为新的 Token 账户
// Token 账户（也叫 Token Account）是用户持有某种代币的账户
// 注意：一个 Mint 可以对应无数个 Token 账户，每个用户可以有多个 Token 账户来持有同一种代币
const token = Keypair.generate();

// 计算创建 Token 账户需要租赁多少 lamports
// Token 账户也需要存储足够的 lamports 才能免于被删除
// 这个数量基于 Token 账户所需的空间大小（ACCOUNT_SIZE）
const tokenRent = await getMinimumBalanceForRentExemptAccount(connection);

// 创建一个"创建账户"的系统指令
// 这将在 Solana 区块链上创建一个新的账户，用于存储 Token 账户数据
const createAccountInstruction = SystemProgram.createAccount({
    fromPubkey: feePayer.publicKey,     // 谁支付创建账户的费用（交易费和租金）
    newAccountPubkey: token.publicKey,   // 新账户的公钥地址
    space: ACCOUNT_SIZE,                 // 新账户分配多少空间（ACCOUNT_SIZE 是 Token 账户的固定大小）
    lamports: tokenRent,                 // 向新账户转移多少 lamports 作为租金
    programId: TOKEN_PROGRAM_ID         // 新账户由哪个程序拥有（这里是 Token Program）
});

// 创建一个"初始化 Token 账户"指令
// 这个指令会设置 Token 账户的初始参数，将它与特定的 Mint 和所有者关联起来
const initializeTokenInstruction = createInitializeAccount3Instruction(
    token.publicKey,        // Token 账户的地址（要初始化的账户）
    mint.publicKey,         // Mint 账户的地址（这个 Token 账户持有的是哪种代币）
    feePayer.publicKey,     // Owner（所有者）：谁拥有这个 Token 账户，只有这个人可以转账其中的代币
    TOKEN_PROGRAM_ID        // 使用 Token Program 来执行这个操作
);

// 创建一个新的交易，并将两条指令添加到交易中
// 交易必须按顺序执行：先创建账户，再初始化账户
const transaction = new Transaction().add(
    createAccountInstruction,      // 第 1 步：创建账户
    initializeTokenInstruction,     // 第 2 步：初始化 Token 账户数据
);

// 发送交易到 Solana 网络并等待确认
// 需要提供两个签名者：
// - keypair (feePayer)：支付交易费用的人
// - token：新账户的创建者（需要签名来证明账户所有权）
const signature = await sendAndConfirmTransaction(connection, transaction, [keypair, token]);

// 在控制台打印交易签名，并生成 Solana Explorer 的链接
// 你可以通过这个链接查看交易的详细信息
console.log(`Token created! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);