// =============================================================================
// Take 指令 - Pinocchio 版本
// =============================================================================
// 本指令用于接受一个现有的托管交易
// 接受者向创建者发送代币 B，并从金库中获得代币 A
//
// 与 Anchor 版本的对应关系见下方各部分注释

use pinocchio::{Address, AccountView, ProgramResult};
use pinocchio::cpi::{Seed, Signer};
use pinocchio::error::ProgramError;
use pinocchio_token::instructions::{CloseAccount, Transfer};
use crate::{AccountCheck, SignerAccount, MintInterface, AssociatedTokenAccount, AssociatedTokenAccountCheck, ProgramAccount, AssociatedTokenAccountInit, Escrow, AccountClose};

// =============================================================================
// TakeAccounts 账户结构体
// =============================================================================
// 对应 Anchor 中的 Take<'info> 结构体
//
// Anchor 版本（take_anchor.rs:9-65）：
//   #[derive(Accounts)]
//   pub struct Take<'info> {
//       #[account(mut)] pub taker: Signer<'info>,
//       #[account(mut)] pub maker: SystemAccount<'info>,
//       #[account(mut, close = maker, seeds = [...], bump = escrow.bump,
//                has_one = maker, has_one = mint_a, has_one = mint_b)]
//       pub escrow: Box<Account<'info, Escrow>>,
//       pub mint_a: Box<InterfaceAccount<'info,Mint>>,
//       pub mint_b: Box<InterfaceAccount<'info,Mint>>,
//       pub vault: Box<InterfaceAccount<'info,TokenAccount>>,
//       #[account(init_if_needed, ...)] pub taker_ata_a: Box<...>,
//       #[account(init_if_needed, ...)] pub taker_ata_b: Box<...>,
//       #[account(init_if_needed, ...)] pub maker_ata_b: Box<...>,
//       pub associated_token_program: Program<'info, AssociatedToken>,
//       pub token_program: Interface<'info, TokenInterface>,
//       pub system_program: Program<'info, System>,
//   }
//
// Pinocchio 版本差异：
// - 使用生命周期参数 'info
// - 每个字段都是 &AccountView 引用
// - 不使用 Box 包装（手动管理借用）
pub struct TakeAccounts<'info> {
    // 接受者账户（签名者）
    // 对应 Anchor: #[account(mut)] pub taker: Signer<'info>
    pub taker: &'info AccountView,

    // 创建者账户（不需要签名）
    // 对应 Anchor: #[account(mut)] pub maker: SystemAccount<'info>
    pub maker: &'info AccountView,

    // 托管账户（PDA，将被关闭）
    // 对应 Anchor: #[account(mut, close = maker, seeds = [...],
    //            bump = escrow.bump, has_one = maker @ EscrowError::InvalidMaker,
    //            has_one = mint_a @ EscrowError::InvalidMintA,
    //            has_one = mint_b @ EscrowError::InvalidMintB)]
    //            pub escrow: Box<Account<'info, Escrow>>
    //
    // has_one 约束说明：
    // - has_one = maker: 验证 escrow.maker == maker.key()
    // - has_one = mint_a: 验证 escrow.mint_a == mint_a.key()
    // - has_one = mint_b: 验证 escrow.mint_b == mint_b.key()
    pub escrow: &'info AccountView,

    // 代币 A 的 Mint 账户
    // 对应 Anchor: pub mint_a: Box<InterfaceAccount<'info,Mint>>
    pub mint_a: &'info AccountView,

    // 代币 B 的 Mint 账户
    // 对应 Anchor: pub mint_b: Box<InterfaceAccount<'info,Mint>>
    pub mint_b: &'info AccountView,

    // 金库账户（将被关闭）
    // 对应 Anchor: #[account(mut, associated_token::mint = mint_a,
    //            associated_token::authority = escrow,
    //            associated_token::token_program = token_program)]
    //            pub vault: Box<InterfaceAccount<'info,TokenAccount>>
    pub vault: &'info AccountView,

    // 接受者的代币 A ATA（可能不存在）
    // 对应 Anchor: #[account(init_if_needed, payer = taker,
    //            associated_token::mint = mint_a,
    //            associated_token::authority = taker, ...)]
    //            pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>
    pub taker_ata_a: &'info AccountView,

    // 接受者的代币 B ATA（用于发送代币给创建者）
    // 对应 Anchor: #[account(init_if_needed, payer = taker,
    //            associated_token::mint = mint_b,
    //            associated_token::authority = taker, ...)]
    //            pub taker_ata_b: Box<InterfaceAccount<'info,TokenAccount>>
    pub taker_ata_b: &'info AccountView,

    // 创建者的代币 B ATA（可能不存在）
    // 对应 Anchor: #[account(init_if_needed, payer = taker,
    //            associated_token::mint = mint_b,
    //            associated_token::authority = maker, ...)]
    //            pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>
    pub maker_ata_b: &'info AccountView,

    // 系统程序
    // 对应 Anchor: pub system_program: Program<'info, System>
    pub system_program: &'info AccountView,

    // 代币程序
    // 对应 Anchor: pub token_program: Interface<'info, TokenInterface>
    pub token_program: &'info AccountView,
}

// =============================================================================
// TryFrom 实现 - 账户解析与验证
// =============================================================================
// 对应 Anchor 的 #[account(...)] 约束验证
impl<'info> TryFrom<&'info [AccountView]> for TakeAccounts<'info> {
    type Error = ProgramError;

    // 从账户数组中解析和验证账户
    // 对应 Anchor 自动进行的账户验证
    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        // 解构账户数组
        // 对应 Anchor 自动按字段名顺序解析账户
        let [taker, maker, escrow, mint_a, mint_b, vault, taker_ata_a, taker_ata_b, maker_ata_b, system_program, token_program, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // =====================================================================
        // 账户验证
        // =====================================================================
        // 对应 Anchor 的各种 #[account(...)] 约束

        // 验证 taker 是签名者
        // 对应 Anchor: pub taker: Signer<'info>
        SignerAccount::check(taker)?;

        // 验证 escrow 是本程序拥有的账户
        // 对应 Anchor: pub escrow: Box<Account<'info, Escrow>>
        // Account<T> 自动验证 owner 和数据长度
        ProgramAccount::check(escrow)?;

        // 验证 mint_a 是有效的 Mint 账户
        // 对应 Anchor: pub mint_a: Box<InterfaceAccount<'info,Mint>>
        MintInterface::check(mint_a)?;

        // 验证 mint_b 是有效的 Mint 账户
        // 对应 Anchor: pub mint_b: Box<InterfaceAccount<'info,Mint>>
        MintInterface::check(mint_b)?;

        // 验证 taker_ata_b 是正确的 ATA
        // 对应 Anchor: #[account(init_if_needed, payer = taker,
        //            associated_token::mint = mint_b,
        //            associated_token::authority = taker, ...)]
        // 注意：这里只验证，不创建（创建在后续的 init_if_needed 中）
        AssociatedTokenAccount::check(taker_ata_b, taker, mint_b, token_program)?;

        // 验证 vault 是正确的 ATA（由 escrow 拥有）
        // 对应 Anchor: #[account(mut, associated_token::mint = mint_a,
        //            associated_token::authority = escrow, ...)]
        AssociatedTokenAccount::check(vault, escrow, mint_a, token_program)?;

        // 注意：taker_ata_a 和 maker_ata_b 不在这里验证
        // 因为它们可能不存在，会在 init_if_needed 中处理

        // 返回验证通过的账户结构
        Ok(Self {
            taker,
            maker,
            escrow,
            mint_a,
            mint_b,
            taker_ata_a,
            taker_ata_b,
            maker_ata_b,
            vault,
            system_program,
            token_program,
        })
    }
}

// =============================================================================
// Take 指令主结构体
// =============================================================================
// 对应 Anchor 的 Context<Take>
pub struct Take<'info> {
    pub accounts: TakeAccounts<'info>,
}

// =============================================================================
// TryFrom 实现 - 指令完整解析与账户初始化
// =============================================================================
// 对应 Anchor 的 Context 解析 + init_if_needed 约束处理
impl<'info> TryFrom<&'info [AccountView]> for Take<'info> {
    type Error = ProgramError;

    // 从账户数组中解析完整的指令
    // 对应 Anchor 自动进行的：
    // 1. 账户验证（#[account] 宏）
    // 2. init_if_needed 约束处理（如果账户不存在则创建）
    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        // 步骤 1: 解析和验证账户
        // 对应 Anchor 的账户验证阶段
        let accounts = TakeAccounts::try_from(accounts)?;

        // =====================================================================
        // 条件账户初始化
        // =====================================================================
        // 对应 Anchor 的 init_if_needed 约束
        //
        // Anchor 版本（take_anchor.rs:88-98）：
        //   #[account(
        //       init_if_needed,           // ← 如果账户不存在则创建
        //       payer = taker,            // ← taker 支付创建费用
        //       associated_token::mint = mint_a,
        //       associated_token::authority = taker,
        //       associated_token::token_program = token_program
        //   )]
        //   pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,
        //
        // Pinocchio 手动实现：
        // 1. 先尝试验证账户是否存在
        // 2. 如果不存在，则创建新账户

        // 创建接受者的代币 A ATA（如果不存在）
        // 对应 Anchor: pub taker_ata_a 的 init_if_needed 约束
        // helpers.rs 中的 init_if_needed 实现：
        // - 先尝试验证账户（check）
        // - 如果验证失败，说明账户不存在，调用 init 创建
        AssociatedTokenAccount::init_if_needed(
            accounts.taker_ata_a,     // 要创建/验证的账户
            accounts.mint_a,          // mint 账户
            accounts.taker,           // payer：对应 Anchor 的 payer = taker
            accounts.taker,           // owner：对应 Anchor 的 authority = taker
            accounts.system_program,  // System Program
            accounts.token_program,   // Token Program
        )?;

        // 创建创建者的代币 B ATA（如果不存在）
        // 对应 Anchor: pub maker_ata_b 的 init_if_needed 约束
        // （take_anchor.rs:112-119）
        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_b,     // 要创建/验证的账户
            accounts.mint_b,          // mint 账户
            accounts.taker,           // payer：对应 Anchor 的 payer = taker
            accounts.maker,           // owner：对应 Anchor 的 authority = maker
            accounts.system_program,  // System Program
            accounts.token_program,   // Token Program
        )?;

        // 返回完整的指令结构
        Ok(Self {
            accounts,
        })
    }
}

impl<'a> Take<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        let data = self.accounts.escrow.try_borrow_data()?;
        let escrow = Escrow::load(&data)?;

        // Check if the escrow is valid
        let escrow_key = create_program_address(&[b"escrow", self.accounts.maker.key(), &escrow.seed.to_le_bytes(), &escrow.bump], &crate::ID)?;
        if &escrow_key != self.accounts.escrow.key() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let seed_binding = escrow.seed.to_le_bytes();
        let bump_binding = escrow.bump;
        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(self.accounts.maker.key().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump_binding),
        ];
        let signer = Signer::from(&escrow_seeds);

        let amount = TokenAccount::from_account_info(self.accounts.vault)?.amount();

        // Transfer from the Vault to the Taker
        Transfer {
            from: self.accounts.vault,
            to: self.accounts.taker_ata_a,
            authority: self.accounts.escrow,
            amount,
        }.invoke_signed(&[signer.clone()])?;

        // Close the Vault
        CloseAccount {
            account: self.accounts.vault,
            destination: self.accounts.maker,
            authority: self.accounts.escrow,
        }.invoke_signed(&[signer.clone()])?;

        // Transfer from the Taker to the Maker
        Transfer {
            from: self.accounts.taker_ata_b,
            to: self.accounts.maker_ata_b,
            authority: self.accounts.taker,
            amount: escrow.receive,
        }.invoke()?;

        // Close the Escrow
        drop(data);
        ProgramAccount::close(self.accounts.escrow, self.accounts.taker)?;

        Ok(())
    }
}
