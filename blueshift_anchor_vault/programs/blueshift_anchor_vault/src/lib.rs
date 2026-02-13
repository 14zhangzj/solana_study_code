use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("22222222222222222222222222222222222222222222");

#[program]
pub mod blueshift_anchor_vault {
    use super::*;

    pub fn deposit(ctx: Context<VaultAction>, amount: u64) -> Result<()> {
        // deposit logic
        //验证金库为空（lamports 为零），以防止重复存款
        require_eq!(ctx.accounts.vault.lamports(), 0, VaultError::VaultAlreadyExists);

        // 确保存款金额超过 SystemAccount 的免租金最低限额
        require_gt!(amount, Rent::get()?.minimum_balance(0), VaultError::InvalidAmount);
        // 使用 CPI 调用系统程序，将 lamports 从签名者转移到金库
        //准备转账单
        let cpi_accounts = Transfer {
            from:ctx.accounts.signer.to_account_info(),
            to:ctx.accounts.vault.to_account_info()
        };

        //准备转账程序
        let cpi_program = ctx.accounts.system_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<VaultAction>) -> Result<()> {
        // withdraw logic
        require_neq!(ctx.accounts.vault.lamports(), 0, VaultError::InvalidAmount);

        //校验调用者的身份，框架已经帮我们完成了
        let seeds = &[
            b"vault".as_ref(),
            ctx.accounts.signer.to_account_info().key.as_ref(),
            &[ctx.bumps.vault],
        ];

        let signer = &[&seeds[..]];

        //准备转账单
        let cpi_accounts = Transfer {
            from:ctx.accounts.vault.to_account_info(),
            to:ctx.accounts.signer.to_account_info()
        };

        let cpi_program = ctx.accounts.system_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        transfer(cpi_ctx, ctx.accounts.vault.lamports())?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct VaultAction<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", signer.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum VaultError {
    #[msg("Vault already exists")]
    VaultAlreadyExists,
    #[msg("Invalid amount")]
    InvalidAmount,
}