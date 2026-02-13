// 从 @solana/web3.js 导入所需的 Solana 核心功能
import {
    sendAndConfirmTransaction,     // 发送交易并等待确认的函数
    Transaction,                   // 交易类，用于打包和发送指令
} from "@solana/web3.js";

// 从 @solana/spl-token 导入 SPL Token 相关的功能
import {
    TOKEN_PROGRAM_ID,                           // Token Program 的程序 ID
    createAssociatedTokenAccountIdempotentInstruction, // 创建关联代币账户的指令（幂等版本）
    getAssociatedTokenAddress,                  // 计算关联代币账户地址的函数
} from "@solana/spl-token";

// 计算（推导）关联代币账户（ATA）的地址
// ATA 的地址不是随机生成的，而是可以通过算法确定的派生地址（PDA - Program Derived Address）
// 关键特性：对于同一个 Mint 和同一个 Owner，ATA 地址永远是相同的
const associatedTokenAccount = await getAssociatedTokenAddress(
    mint.publicKey,      // Mint 账户的公钥（持有哪种代币）
    keypair.publicKey,   // Owner 的公钥（谁拥有这个 ATA）
    false,               // 是否允许 Owner 是"off-curve"账户（即没有私钥的账户，如 PDA）
                         // false 表示 Owner 必须是普通的 Ed25519 密钥对（有私钥）
                         // true 表示 Owner 可以是程序派生地址（PDA），用于程序拥有代币的场景
    TOKEN_PROGRAM_ID     // 使用 Token Program（也可以用 Token-2022 Program）
);

// 创建一个"创建关联代币账户"的指令
// 使用 Idempotent（幂等）版本的好处：如果账户已经存在，不会报错，而是直接成功
// 相比之下，非幂等版本会在账户存在时报错
const createAtaInstruction = createAssociatedTokenAccountIdempotentInstruction(
    keypair.publicKey,         // Payer：谁支付创建 ATA 的费用（交易费 + 租金）
    associatedTokenAccount,    // ATA 的地址（这个地址是上面计算出来的）
    keypair.publicKey,         // Owner：谁拥有这个 ATA（只有这个地址可以转账其中的代币）
    mint.publicKey,            // Mint：这个 ATA 持有的是哪种代币
    TOKEN_PROGRAM_ID           // 使用 Token Program 来执行这个操作
);

// 创建一个新的交易，并将 ATA 创建指令添加到交易中
// 注意：只需要一条指令！ATA 的创建是原子操作，不需要"创建账户"和"初始化账户"两个步骤
const transaction = new Transaction().add(
    createAtaInstruction,      // 创建 ATA（如果已存在则跳过）
);

// 发送交易到 Solana 网络并等待确认
// 只需要一个签名者：keypair (payer & owner)
// 不需要 ATA 的签名者，因为 ATA 是派生地址（PDA），没有私钥
const signature = await sendAndConfirmTransaction(connection, transaction, [keypair]);

// 在控制台打印交易签名，并生成 Solana Explorer 的链接
// 你可以通过这个链接查看交易的详细信息
console.log(`Associated Token created! Check out your TX here: https://explorer.solana.com/tx/${signature}?cluster=devnet`);