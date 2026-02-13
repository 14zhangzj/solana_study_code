// ==================== 导入依赖模块 ====================
// 导入 Anchor 框架的所有功能，用 * as anchor 方式导入，可以通过 anchor.xxx 使用
import * as anchor from "@coral-xyz/anchor";
// 导入 Program 类型，用于类型注解
import { Program } from "@coral-xyz/anchor";
// 导入自动生成的 TypeScript 类型定义，从 target/types/ 目录导入
import { PxsolSsAnchor } from "../target/types/pxsol_ss_anchor";

// ==================== 测试套件开始 ====================
// Mocha 测试框架的语法，定义一个测试套件（test suite）
describe("pxsol-ss-anchor", () => {
  // 配置客户端使用本地集群
  // AnchorProvider.env() 从环境变量读取配置：
  //   - ANCHOR_PROVIDER_URL：RPC 节点地址（默认本地）
  //   - ANCHOR_WALLET：钱包文件路径（默认 ~/.config/solana/id.json）
  anchor.setProvider(anchor.AnchorProvider.env());

  // ==================== 获取程序实例 ====================
  // anchor.workspace 是 Anchor 自动创建的工作空间对象
  // pxsolSsAnchor 对应 Anchor.toml 中定义的程序名称
  // 这个 program 对象包含了所有可以调用的指令（methods）
  const program = anchor.workspace.pxsolSsAnchor;

  // ==================== 获取 Provider 和钱包 ====================
  // 获取当前设置的 Provider，Provider 是连接到 Solana 网络的接口
  const provider = anchor.getProvider() as anchor.Provider;

  // 从 Provider 中提取钱包（Wallet）
  // 钱包包含：
  //   - publicKey：公钥
  //   - payer：签名者（用于支付交易费用和签名交易）
  const wallet = provider.wallet as anchor.Wallet;

  // ==================== 计算 PDA 地址 ====================
  // 使用同步方法查找 PDA（Program Derived Address）
  // PDA 是从程序 ID 和种子派生出的地址，在链外可以计算，但没有私钥
  const walletPda = anchor.web3.PublicKey.findProgramAddressSync(
    // 定义种子（seeds）数组，必须和 Rust 程序中定义的完全一致：
    [Buffer.from("data"), wallet.publicKey.toBuffer()],
    // 传入程序 ID（programId），PDA 是相对于程序 ID 计算的
    program.programId
  )[0]; // 取返回值的第一个元素（PDA 地址），findProgramAddressSync 返回 [publicKey, bump]

  // ==================== 测试用例开始 ====================
  // it() 定义一个测试用例
  // 测试描述：初始化包含内容，然后更新（增长和收缩）
  it("Init with content and then update ( grow and shrink )", async () => {
    // ==================== 空投 SOL ====================
    // 确保钱包有足够的 SOL 支付交易费用
    // requestAirdrop：请求空投 SOL 到钱包地址
    // confirmTransaction：等待交易确认
    await provider.connection.confirmTransaction(await provider.connection.requestAirdrop(
      wallet.publicKey,      // 目标地址（钱包的公钥）
      10 * anchor.web3.LAMPORTS_PER_SOL  // 空投数量：10 SOL（1 SOL = 1,000,000,000 lamports）
    ), "confirmed");  // 确认级别设置为 "confirmed"（已确认）

    // ==================== 准备测试数据 ====================
    // 定义三个 Buffer 变量用于测试账户空间的增长和收缩
    const poemInitial = Buffer.from("");  // 空数据（初始状态）
    const poemEnglish = Buffer.from("The quick brown form jumps over the lazy dog");  // 英文（45 字节）
    const poemChinese = Buffer.from("片云天共远，永夜月同孤");  // 中文（UTF-8 编码，30 字节）

    // ==================== 定义辅助函数 ====================
    // 定义一个异步箭头函数，用于从链上读取 PDA 账户的数据
    const walletPdaData = async (): Promise<Buffer<ArrayBuffer>> => {
      // program.account.data.fetch() 从链上获取 PDA 账户的数据
      // program.account.data 是自动生成的账户类型访问器
      let walletPdaData = await program.account.data.fetch(walletPda);
      // 从获取的数据中提取 data 字段并转为 Buffer
      return Buffer.from(walletPdaData.data);
    };

    // ==================== 调用 initialize 指令 ====================
    // program.methods 指定要调用的指令
    await program.methods
      .initialize()  // 调用 initialize 指令（不需要参数）
      .accounts({    // 指定指令需要的账户
        user: wallet.publicKey,  // 用户公钥（对应 Rust 程序中的 user 账户）
        userPda: walletPda,      // PDA 地址（对应 user_pda 账户）
        systemProgram: anchor.web3.SystemProgram.programId  // 系统程序（用于创建账户）
      })
      .signers([wallet.payer])  // 指定签名者（需要钱包签名来证明账户所有权）
      .rpc();  // 发送交易到 Solana 网络

    // ==================== 验证初始数据 ====================
    // 调用辅助函数获取链上数据，验证是否为空
    if (!(await walletPdaData()).equals(poemInitial)) {
      throw new Error("Unable to initialize payer");
    }

    // ==================== 调用 update 指令（增长账户空间） ====================
    await program.methods
      .update(poemEnglish)  // 传递英文数据作为参数（45 字节）
      .accounts({
        user: wallet.publicKey,
        userPda: walletPda,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([wallet.payer])
      .rpc();

    // ==================== 验证英文数据 ====================
    // 验证链上数据是否等于英文数据
    if (!(await walletPdaData()).equals(poemEnglish)) {
      throw new Error("Unable to update to English poem");
    }

    // ==================== 调用 update 指令（收缩账户空间） ====================
    await program.methods
      .update(poemChinese)  // 传递中文数据作为参数（30 字节，账户从 45 字节收缩到 30 字节）
      .accounts({
        user: wallet.publicKey,
        userPda: walletPda,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([wallet.payer])
      .rpc();  // 发送交易到 Solana 网络

    // ==================== 验证中文数据 ====================
    // 验证链上数据是否等于中文数据
    if (!(await walletPdaData()).equals(poemChinese)) {
      throw new Error("Unable to update to Chinese poem");
    }
  });
});
