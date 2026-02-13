# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于 Anchor 框架开发的 Solana 托管（Escrow）程序，实现无需信任的代币交换功能。程序允许两方安全地交换不同类型的 SPL 代币，无需双方同时在线。

## 核心功能

程序通过三个指令实现代币托管交换：

1. **Make**：创建者（maker）锁定代币 A 到保险库，并指定期望接收的代币 B 数量
2. **Take**：接受者（taker）存入代币 B 给创建者，并获得锁定的代币 A
3. **Refund**：创建者取消交易并取回锁定的代币 A

## 开发命令

### 构建和测试

```bash
# 构建程序
anchor build

# 运行所有测试
anchor test

# 运行测试并保留测试容器
anchor test --skip-local-validator

# 代码格式化
yarn run lint:fix

# 检查代码格式
yarn run lint
```

### 单个测试

修改 `tests/blueshift_anchor_escrow.ts` 中的 `describe` 或 `it` 块，然后运行：

```bash
# 仅运行匹配的测试
anchor test --skip-local-validator -- test_name
```

### 部署到本地网络

```bash
# 启动本地验证器（在另一个终端）
solana-test-validator

# 部署程序
anchor deploy
```

## 架构

### 模块结构

```
programs/blueshift_anchor_escrow/src/
├── lib.rs              # 程序入口，包含 #[program] 宏和指令分发
├── state.rs            # Escrow 账户数据结构定义
├── errors.rs           # 自定义错误类型
└── instructions/
    ├── mod.rs          # 指令模块导出
    ├── make.rs         # Make 指令实现
    ├── take.rs         # Take 指令实现
    └── refund.rs       # Refund 指令实现
```

### 关键设计模式

#### 1. 指令处理分离

每个指令的实现逻辑在 `instructions/` 下的独立模块中，通过 `handler()` 函数暴露：

```rust
// lib.rs
#[program]
pub mod blueshift_anchor_escrow {
    pub fn make(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
        instructions::make::handler(ctx, seed, receive, amount);
        Ok(())
    }
}

// instructions/make.rs
pub fn handler(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
    // 实际实现逻辑
}
```

#### 2. Accounts 结构体导出

在 `instructions/mod.rs` 中使用 `pub use` 重新导出每个指令的 Accounts 结构体，使它们在 `#[program]` 模块中可用：

```rust
pub use make::Make;
pub use take::Take;
pub use refund::Refund;
```

#### 3. PDA (Program Derived Address) 签名

程序使用 PDA 作为托管账户的权限。在需要 PDA 签名时（如从 vault 转出代币），使用 `CpiContext::new_with_signer()`：

```rust
let signer_seeds: [&[&[u8]]; 1] = [&[
    b"escrow",
    maker.key.as_ref(),
    &escrow.seed.to_le_bytes()[..],
    &[escrow.bump],
]];

transfer_checked(
    CpiContext::new_with_signer(
        token_program.to_account_info(),
        TransferChecked { ... },
        &signer_seeds,
    ),
    amount,
    decimals,
)?;
```

### Escrow 状态结构

`state::Escrow` 账户存储以下关键字段：

- `seed: u64` - PDA 派生种子，允许同一创建者创建多个托管
- `maker: Pubkey` - 创建者公钥，用于验证权限
- `mint_a: Pubkey` - 创建者存入的代币类型
- `mint_b: Pubkey` - 创建者期望接收的代币类型
- `receive: u64` - 期望接收的代币 B 数量
- `bump: u8` - 缓存的 PDA bump seed

### 使用 `#[instruction(...)]` 宏

某些指令需要在 Accounts 验证阶段访问指令参数。使用 `#[instruction(...)]` 声明参数：

```rust
#[derive(Accounts)]
#[instruction(seed: u64)]  // 声明需要使用 seed 参数
pub struct Make<'info> {
    #[account(
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,
}
```

**重要**：参数名不需要与指令函数参数名匹配，Anchor 按位置匹配参数。

### CPI (Cross-Program Invocation)

程序使用 `transfer_checked` 而非 `transfer` 进行代币转账，以验证 mint 和精度：

```rust
// 普通账户签名
transfer_checked(
    CpiContext::new(
        token_program.to_account_info(),
        TransferChecked { ... },
    ),
    amount,
    decimals,
)?;

// PDA 签名
transfer_checked(
    CpiContext::new_with_signer(
        token_program.to_account_info(),
        TransferChecked { ... },
        &signer_seeds,
    ),
    amount,
    decimals,
)?;
```

### 关键约束使用

- `has_one`: 验证 Escrow 账户中的字段与传入的账户匹配
- `associated_token::*`: 验证 ATA 的 mint、authority 和 token_program
- `init_if_needed`: 如果账户不存在则创建（谨慎使用）
- `close = maker`: 关闭账户并将租金退还给 maker

## TypeScript 测试

测试文件位于 `tests/blueshift_anchor_escrow.ts`。使用 Anchor 的 TypeScript 框架编写测试：

```typescript
const program = anchor.workspace.blueshiftAnchorEscrow as Program<BlueshiftAnchorEscrow>;

// 调用指令
await program.methods
  .make(seed, receive, amount)
  .accounts({ /* ... */ })
  .rpc();
```

## 注意事项

1. **Anchor 版本**：项目使用 Anchor 0.32.1，确保所有依赖项版本匹配
2. **命名规范**：
   - 模块名使用 snake_case（如 `make.rs`）
   - 结构体名使用 PascalCase（如 `Make`）
   - 函数名使用 snake_case
3. **接口类型**：使用 `InterfaceAccount` 和 `TokenInterface` 以支持 Token-2022
4. **Box 包装**：大型账户结构使用 `Box<Account>` 或 `Box<InterfaceAccount>` 以减少栈空间使用