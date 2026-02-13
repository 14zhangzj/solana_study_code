// 从 @solana/web3.js 导入所需的 Solana 核心功能
import {
    Keypair,                      // 密钥对类，用于生成公钥和私钥
    sendAndConfirmTransaction,     // 发送交易并等待确认的函数
    SystemProgram,                 // 系统程序，用于创建账户等系统级操作
    Transaction,                   // 交易类，用于打包和发送指令
} from "@solana/web3.js";

// 从 @solana/spl-token 导入 SPL Token 相关的功能
import {
    createInitializeMint2Instruction, // 创建初始化 Mint 账户的指令
    MINT_SIZE,                        // Mint 账户所需的空间大小（字节）
    getMinimumBalanceForRentExemptMint, // 获取 Mint 账户免租金所需的最低 lamports
    TOKEN_PROGRAM_ID,                 // Token Program 的程序 ID
} from "@solana/spl-token";

// 生成一个新的随机密钥对，这个将作为新的 Mint 账户
// Mint 账户是 SPL Token 的核心，代表一种代币的"铸币厂"
const mint = Keypair.generate();

// 计算创建 Mint 账户需要租赁多少 lamports
// 在 Solana 上，账户必须存储足够的 lamports 才能免于被删除（租金豁免）
// 这个数量基于账户所需的空间大小（MINT_SIZE）
const mintRent = await getMinimumBalanceForRentExemptMint(connection);

// 创建一个"创建账户"的系统指令
// 这将在 Solana 区块链上创建一个新的账户，用于存储 Mint 数据
const createAccountInstruction = SystemProgram.createAccount({
    fromPubkey: feePayer.publicKey,     // 谁支付创建账户的费用（交易费和租金）
    newAccountPubkey: mint.publicKey,   // 新账户的公钥地址
    space: MINT_SIZE,                   // 新账户分配多少空间（MINT_SIZE 是 SPL Token 定义的固定大小）
    lamports: mintRent,                 // 向新账户转移多少 lamports 作为租金
    programId: TOKEN_PROGRAM_ID         // 新账户由哪个程序拥有（这里是 Token Program）
});

// 创建一个"初始化 Mint"指令
// 这个指令会设置 Mint 账户的初始参数，让它成为一个可用的代币 Mint
const initializeMintInstruction = createInitializeMint2Instruction(
    mint.publicKey,        // Mint 账户的地址（要初始化的账户）
    6,                     // 代币的小数位数（6 表示像 USDC 一样，1 个代币 = 1,000,000 最小单位）
    feePayer.publicKey,    // Mint Authority（铸币权限）：谁有权铸造新代币
    null,                  // Freeze Authority（冻结权限）：谁有权冻结代币账户，null 表示没有这个权限
    TOKEN_PROGRAM_ID       // 使用 Token Program 来执行这个操作
);

// 创建一个新的交易，并将两条指令添加到交易中
// 交易必须按顺序执行：先创建账户，再初始化账户
const transaction = new Transaction().add(
    createAccountInstruction,      // 第 1 步：创建账户
    initializeMintInstruction,     // 第 2 步：初始化 Mint 数据
);

// 发送交易到 Solana 网络并等待确认
// 需要提供两个签名者：
// - keypair (feePayer)：支付交易费用的人
// - mint：新账户的创建者（需要签名来证明账户所有权）
const signature = await sendAndConfirmTransaction(connection, transaction, [keypair, mint]);

// 在控制台打印交易签名，并生成 Solana Explorer 的链接
// 你可以通过这个链接查看交易的详细信息
console.log(`Mint created! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);