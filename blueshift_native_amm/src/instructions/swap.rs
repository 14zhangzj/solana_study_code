use constant_product_curve::{ConstantProduct, LiquidityPair};
use pinocchio::{AccountView, ProgramResult};
use pinocchio::cpi::{Seed, Signer};
use pinocchio::error::ProgramError;
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::Sysvar;
use pinocchio_token::instructions::Transfer;
use pinocchio_token::state::TokenAccount;
use crate::Config;

pub struct SwapAccounts<'info> {
    pub user: &'info AccountView,
    pub user_x_ata: &'info AccountView,
    pub user_y_ata: &'info AccountView,
    pub vault_x: &'info AccountView,
    pub vault_y: &'info AccountView,
    pub config: &'info AccountView,
    pub token_program: &'info AccountView,
}
impl<'info> TryFrom<&'info [AccountView]> for SwapAccounts<'info> {
    type Error = ProgramError;
    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {

        // let mut iter = accounts.iter();
        // Ok(Self {
        //     user: iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?,
        //     user_x_ata: iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?,
        //     user_y_ata: iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?,
        //     vault_x: iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?,
        //     vault_y: iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?,
        //     config: iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?,
        //     token_program: iter.next().ok_or(ProgramError::NotEnoughAccountKeys)?,
        // })
        // ═══════════════════════════════════════════════════════════════════════
        // 数组切片模式解析账户（推荐写法）
        // ═══════════════════════════════════════════════════════════════════════
        // 优点:
        // 1. 编译时验证: 模式匹配会在编译时检查账户数量是否正确
        // 2. 一次性完成: 单次模式匹配完成所有账户解析，无需迭代器状态管理
        // 3. 性能更优: 避免创建迭代器和多次 next() 调用的开销
        // 4. 可读性强: 一眼看出账户结构，代码简洁清晰
        // 5. 易于维护: 添加/删除字段时，编译器会提醒更新模式匹配
        // ═══════════════════════════════════════════════════════════════════════
        let [user, user_x_ata, user_y_ata, vault_x, vault_y, config, token_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self {
            user,
            user_x_ata,
            user_y_ata,
            vault_x,
            vault_y,
            config,
            token_program
        })
    }
}

#[repr(C,packed)]
#[derive(Copy, Clone)]
pub struct SwapInstructionData {
    pub is_x: bool,
    pub amount: u64,
    pub min: u64,
    pub expirations: i64,
}
impl<'info> TryFrom<&'info [u8]> for SwapInstructionData {
    type Error = ProgramError;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(unsafe { *(data.as_ptr() as *const Self) })
    }
}

pub struct Swap<'info> {
    pub accounts: SwapAccounts<'info>,
    pub instruction_data: SwapInstructionData,
}
impl<'info> TryFrom<(&'info [u8], &'info [AccountView])> for Swap<'info> {
    type Error = ProgramError;
    fn try_from((data,accounts): (&'info [u8], &'info [AccountView])) -> Result<Self, Self::Error> {
        let accounts = SwapAccounts::try_from(accounts)?;
        let instruction_data = SwapInstructionData::try_from(data)?;

        Ok(Self{
            accounts,
            instruction_data,
        })
    }
}
impl<'info> Swap<'info> {
    pub const DISCRIMINATOR: &'info u8 = &3;
    pub fn process(&mut self) -> ProgramResult {
        let accounts = &mut self.accounts;
        let data = &self.instruction_data;

        //1. 验证过期时间
        let colck = Clock::get()?;
        if colck.unix_timestamp > data.expirations {
            return Err(ProgramError::InvalidArgument);
        };

        // 2. 加载配置和状态
        let config = Config::load(accounts.config)?;
        if config.state() != 1 {
            //必须是Initialized
            return Err(ProgramError::InvalidAccountData);
        };

        // 3.获取金库当前余额并计算交换
        let vault_x = unsafe{ TokenAccount::from_account_view_unchecked(accounts.vault_x)?};
        let vault_y = unsafe{ TokenAccount::from_account_view_unchecked(accounts.vault_y)?};

        // 将 fee 从 [u8; 2]（小端序）转换为 u16
        let fee = u16::from_le_bytes(*config.fee());

        let mut curve = ConstantProduct::init(
            vault_x.amount(),
            vault_y.amount(),
            vault_x.amount(),
            fee,
            None
        ).map_err(|_| ProgramError::ArithmeticOverflow)?;

        let pair = if data.is_x{
            LiquidityPair::X
        }else {
            LiquidityPair::Y
        };
        let swap_result = curve
            .swap(pair,data.amount,data.min)
            .map_err(|_| ProgramError::InvalidArgument)?;

        //4. 准备签名种子(用于从金库转出)
        let seed_binding = config.seed().to_le_bytes();
        let min_x_key = config.mint_x();
        let min_y_key = config.mint_y();
        let bump = config.config_bump();

        let config_seeds = [
            Seed::from(b"config"),
            Seed::from(&seed_binding),
            Seed::from(min_x_key.as_ref()),
            Seed::from(min_y_key.as_ref()),
            Seed::from(bump),
        ];
        let signer = Signer::from(&config_seeds);

        // 5. 执行原子转账
        if data.is_x{

            //X - Y
            Transfer{
                from:accounts.user_x_ata,
                to:accounts.vault_x,
                authority: accounts.user,
                amount:swap_result.deposit,
            }.invoke()?;

            Transfer{
                from:accounts.vault_y,
                to:accounts.user_y_ata,
                authority:accounts.config,
                amount: swap_result.withdraw,
            }.invoke_signed(&[signer])?;
        }else {
            //Y - X
            Transfer{
                from:accounts.user_y_ata,
                to:accounts.vault_y,
                authority: accounts.user,
                amount:swap_result.deposit,
            }.invoke()?;

            Transfer{
                from:accounts.vault_x,
                to:accounts.user_x_ata,
                authority:accounts.config,
                amount: swap_result.withdraw,
            }.invoke_signed(&[signer])?;
        }

        Ok(())
    }
}