# SPL Token Operations with Web3.js

这个项目演示了如何使用 Solana Web3.js 进行各种 SPL Token 操作。

## 项目结构

```
web3js_SPL_token/
├── src/
│   ├── config/           # 配置文件
│   │   └── index.ts      # 网络和连接配置
│   ├── utils/            # 工具函数
│   │   ├── keypair.ts    # 密钥对管理
│   │   ├── logger.ts     # 日志工具
│   │   └── helpers.ts    # 辅助函数
│   ├── operations/       # Token 操作
│   │   ├── types.ts      # 类型定义
│   │   ├── mint.ts       # 铸造操作
│   │   ├── transfer.ts   # 转账操作
│   │   ├── burn.ts       # 销毁操作
│   │   ├── freeze.ts     # 冻结/解冻操作
│   │   ├── approve.ts    # 授权操作
│   │   └── close.ts      # 关闭账户操作
│   └── index.ts          # 主入口文件
├── .keypairs/            # 密钥对存储目录 (自动创建)
├── package.json
├── tsconfig.json
└── README.md
```

## 安装依赖

```bash
npm install
```

## 主要功能

### 1. 铸造 (Mint)
- 创建新的 Token Mint
- 创建 Token 账户
- 铸造代币到账户

### 2. 转账 (Transfer)
- 转移代币到其他账户

### 3. 销毁 (Burn)
- 销毁持有的代币

### 4. 冻结/解冻 (Freeze/Thaw)
- 冻结代币账户
- 解冻代币账户

### 5. 授权 (Approve/Revoke)
- 授权代理人转移代币
- 撤销授权

### 6. 关闭账户 (Close)
- 关闭代币账户并回收租金

## 使用方法

### 编译项目

```bash
npm run build
```

### 运行项目

```bash
npm start
```

### 开发模式 (使用 ts-node)

```bash
npm run dev
```

## 配置

在 [src/config/index.ts](src/config/index.ts) 中修改网络配置和代币参数：

```typescript
export const NETWORK = process.env.NETWORK || 'devnet';
export const TOKEN_CONFIG = {
  decimals: 6,
  name: 'My Token',
  symbol: 'MTK',
};
```

## 环境变量

可以创建 `.env` 文件来设置环境变量：

```env
NETWORK=devnet
RPC_URL=https://api.devnet.solana.com
```

## 注意事项

1. **钱包安全**: 生成的钱包密钥会保存在 `.keypairs/` 目录中，请确保此目录不被提交到版本控制系统。
2. **网络**: 默认使用 devnet 网络，在 mainnet 上操作前请充分测试。
3. **费用**: 所有操作都需要支付 SOL 作为交易费用。
4. **Airdrop**: devnet 网络可以使用 airdrop 获取测试 SOL。

## 相关资源

- [Solana Web3.js 文档](https://solana-labs.github.io/solana-web3.js/)
- [SPL Token 文档](https://spl.solana.com/token)
- [Blueshift 课程](https://learn.blueshift.gg/zh-CN/courses/spl-token-with-web3js/introduction)

## 许可证

MIT
