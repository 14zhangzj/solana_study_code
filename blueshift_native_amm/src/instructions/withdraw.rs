use constant_product_curve::ConstantProduct;
use pinocchio::{AccountView, ProgramResult};
use pinocchio::cpi::{Seed, Signer};
use pinocchio::error::ProgramError;
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::Sysvar;
use pinocchio_token::instructions::{Burn, Transfer};
use pinocchio_token::state::{Mint, TokenAccount};
use crate::Config;

pub struct WithdrawAccounts<'info> {
    pub user: &'info AccountView,
    pub mint_lp: &'info AccountView,
    pub vault_x: &'info AccountView,
    pub vault_y: &'info AccountView,
    pub user_x_ata: &'info AccountView,
    pub user_y_ata: &'info AccountView,
    pub user_lp_ata: &'info AccountView,
    pub config: &'info AccountView,
    pub token_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for WithdrawAccounts<'info> {
    type Error = ProgramError;
    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [user,mint_lp,vault_x,vault_y,user_x_ata,user_y_ata,user_lp_ata,config,token_program] = accounts else{
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self{
            user,
            mint_lp,
            vault_x,
            vault_y,
            user_x_ata,
            user_y_ata,
            user_lp_ata,
            config,
            token_program
        })
    }
}

// 使用 C 内存布局 + 紧凑排列，确保结构体在内存中的布局与字节数组完全一致
// 内存布局: [amount: u64][mint_x: u64][mint_y: u64][expiration: i64] = 32字节
// 这是直接指针转换反序列化的前提条件
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct WithdrawInstructionData{
    pub amount: u64,
    pub mint_x: u64,
    pub mint_y: u64,
    pub expiration: i64,
}
impl<'info> TryFrom<&'info [u8]> for WithdrawInstructionData {
    type Error = ProgramError;
    fn try_from(data: &'info [u8]) -> Result<WithdrawInstructionData, Self::Error> {
        // 安全检查: 确保字节数组足够大，避免读取越界内存
        if data.len() < size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        };

        // ═══════════════════════════════════════════════════════════════════════
        // 旧写法（安全但性能较差）:
        // - 逐字段解析，需要多次切片和字节转换
        // - 每次都有边界检查，消耗更多计算单元(CUs)
        // ═══════════════════════════════════════════════════════════════════════
        // let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
        // let max_x = u64::from_le_bytes(data[8..16].try_into().unwrap());
        // let max_y = u64::from_le_bytes(data[16..24].try_into().unwrap());
        // let expirations = i64::from_le_bytes(data[24..32].try_into().unwrap());
        //
        // Ok(Self{ amount, max_x, max_y, expirations })

        // ═══════════════════════════════════════════════════════════════════════
        // 新写法（性能优化，Solana 标准做法）:
        // - 直接指针转换，一次性读取整个结构体
        // - 配合 #[repr(C, packed)] 属性，确保内存布局匹配
        // - 在 BPF 环境下安全且高效，节省 CUs
        // ═══════════════════════════════════════════════════════════════════════
        Ok(unsafe { *(data.as_ptr() as *const Self) })
    }
}

pub struct Withdraw<'info> {
    pub accounts: WithdrawAccounts<'info>,
    pub instruction_data: WithdrawInstructionData,
}

impl<'info> TryFrom<(&'info [u8],&'info [AccountView])> for Withdraw<'info> {
    type Error = ProgramError;
    fn try_from((data,accounts): (&'info [u8],&'info [AccountView])) -> Result<Self, Self::Error> {
        let accounts = WithdrawAccounts::try_from(accounts)?;
        let instruction_data = WithdrawInstructionData::try_from(data)?;

        Ok(Self{
            accounts,
            instruction_data,
        })
    }
}

impl<'info> Withdraw<'info> {
    pub const DISCRIMINATOR: &'info u8 = &2;
    pub fn process(&mut self) -> ProgramResult {

        let accounts = &self.accounts;
        let data = &self.instruction_data;

        //1. 过期检查
        let clock = Clock::get()?;
        if clock.unix_timestamp > data.expiration{
            return Err(ProgramError::InvalidArgument);
        }

        //2. 加载状态并检查 Withdraw要求非Disable
        let config = Config::load(accounts.config)?;
        if config.state() == 2 {
            return Err(ProgramError::InvalidAccountData);
        }

        //3. 反序列化代币信息
        let mint_lp = unsafe { Mint::from_account_view_unchecked(accounts.mint_lp)? };
        let vault_x = unsafe { TokenAccount::from_account_view_unchecked(accounts.vault_x)? };
        let vault_y = unsafe { TokenAccount::from_account_view_unchecked(accounts.vault_y)? };

        //4. 计算应退还的 X, Y数量
        let (x,y) = if mint_lp.supply() == data.amount{

            //全额提取:直接取走所有余额,防止舍入误差留下“尘埃”
            (vault_x.amount(),vault_y.amount())
        }else {
            let amounts = ConstantProduct::xy_deposit_amounts_from_l(
                vault_x.amount(),
                vault_y.amount(),
                mint_lp.supply(),
                data.amount,
                6
            ).map_err(|_| ProgramError::ArithmeticOverflow)?;
            (amounts.x,amounts.y)
        };

        // 5. 滑点检查
        if x < data.mint_x || y < data.mint_y {
            return Err(ProgramError::InvalidArgument);
        }

        // 6. 销毁用户的LP
        Burn {
            mint: accounts.mint_lp,
            account: accounts.user_lp_ata,
            authority: accounts.user,
            amount: data.amount,
        }.invoke()?;

        // 7. 构造Config PDA签名从金库转账
        let seed_binding = config.seed().to_le_bytes();
        let mint_x_key = config.mint_x();
        let mint_y_key = config.mint_y();
        let bump = config.config_bump();

        let config_seeds = [
            Seed::from(b"config"),
            Seed::from(&seed_binding),
            Seed::from(mint_x_key.as_ref()),
            Seed::from(mint_y_key.as_ref()),
            Seed::from(bump),
        ];
        let signer = Signer::from(&config_seeds);

        // 8. 转移Token X和Y (Config PDA签名)
        Transfer {
            from: accounts.vault_x,
            to: accounts.user_x_ata,
            authority: accounts.config,
            amount: x,
        }.invoke_signed(&[signer.clone()])?;

        Transfer {
            from: accounts.vault_y,
            to: accounts.user_y_ata,
            authority: accounts.config,
            amount: y,
        }.invoke_signed(&[signer.clone()])?;
        Ok(())
    }
}