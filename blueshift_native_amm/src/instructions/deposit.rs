use constant_product_curve::ConstantProduct;
use pinocchio::{AccountView, ProgramResult};
use pinocchio::cpi::{Seed, Signer};
use pinocchio::error::ProgramError;
use pinocchio::sysvars::{clock, Sysvar};
use pinocchio::sysvars::clock::Clock;
use pinocchio_token::instructions::{MintTo, Transfer};
use pinocchio_token::state::{Mint, TokenAccount};
use solana_address::Address;
use crate::Config;

pub struct DepositAccounts<'info>{
    pub user: &'info AccountView,
    pub mint_lp: &'info AccountView,
    pub vault_x:&'info AccountView,
    pub vault_y:&'info AccountView,
    pub user_x_ata:&'info AccountView,
    pub user_y_ata:&'info AccountView,
    pub user_lp_ata:&'info AccountView,
    pub config:&'info AccountView,
    pub token_program:&'info AccountView,
}
impl<'info> TryFrom<&'info [AccountView]> for DepositAccounts<'info> {
    type Error = ProgramError;
    fn try_from(accounts:&'info [AccountView]) -> Result<Self, Self::Error> {
        let [user,
        mint_lp,
        vault_x,
        vault_y,
        user_x_ata,
        user_y_ata,
        user_lp_ata,
        config,
        token_program] = accounts else{
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

pub struct DepositInstructionData{
    pub amount:u64,
    pub max_x:u64,
    pub max_y:u64,
    pub expirations:i64
}

impl<'info> TryFrom<&'info [u8]> for DepositInstructionData{
    type Error = ProgramError;
    fn try_from(data: &'info [u8]) -> Result<Self, Self::Error> {
        if data.len() < size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let max_x = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let max_y = u64::from_le_bytes(data[16..24].try_into().unwrap());
        let expirations = i64::from_le_bytes(data[24..32].try_into().unwrap());

        Ok(Self{
            amount,
            max_x,
            max_y,
            expirations
        })
    }
}

pub struct Deposit<'info>{
    pub accounts: DepositAccounts<'info>,
    pub instruction_data: DepositInstructionData,
}

impl<'info> TryFrom<(&'info [u8],&'info [AccountView])> for Deposit<'info>{
    type Error = ProgramError;
    fn try_from((data,accounts):(&'info [u8],&'info [AccountView])) -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(accounts)?;
        let instruction_data = DepositInstructionData::try_from(data)?;

        Ok(Self{
            accounts,
            instruction_data,
        })
    }
}

impl<'info> Deposit<'info>{
    pub const DISCRIMINATOR: &'info u8 = &1;

    pub fn process(&mut self) -> ProgramResult{
        let accounts = &self.accounts;
        let data = &self.instruction_data;

        //1. 过期检查
        let clock = Clock::get()?;
        if clock.unix_timestamp > data.expirations {
            return Err(ProgramError::InvalidArgument);
        }

        // 2. 加载Config并验证状态
        let config = Config::load(accounts.config)?;
        if config.state() != 1 {
            return Err(ProgramError::InvalidAccountData);
        };

        // 3.反序列化代币账户信息
        let mint_lp = unsafe { Mint::from_account_view_unchecked(accounts.mint_lp)? };
        let vault_x = unsafe { TokenAccount::from_account_view_unchecked(accounts.vault_x)? };
        let vault_y = unsafe { TokenAccount::from_account_view_unchecked(accounts.vault_y)? };

        // 4. 计算存款金额(x,y)
        let (x,y) = if mint_lp.supply() == 0{
            //初始流动性:使用用户指定的 max值
            (data.max_x, data.max_y)
        } else {
            //后续流动性:基于比例计算
            let amounts = ConstantProduct::xy_deposit_amounts_from_l(
                vault_x.amount(),
                vault_y.amount(),
                mint_lp.supply(),
                data.amount,
                6,
            ).map_err(|_| ProgramError::ArithmeticOverflow)?;
            (amounts.x, amounts.y)
        };

        // 5. 滑点保护检查
        if x > data.max_x || y > data.max_y {
            return Err(ProgramError::InvalidArgument);
        }

        // 6. 执行代币转移(用户 -》 金库)
        Transfer{
            from:accounts.user_x_ata,
            to:accounts.vault_x,
            authority:accounts.user,
            amount:x,
        }.invoke()?;

        Transfer{
            from:accounts.user_y_ata,
            to:accounts.vault_y,
            authority:accounts.user,
            amount:y,
        }.invoke()?;

        //7. 签署并执行MintTo(Config PDA -》 用户)
        let seed_binding = config.seed().to_le_bytes();
        let mint_x = config.mint_x();
        let mint_y = config.mint_y();
        let bump = config.config_bump();

        let config_seeds = [
            Seed::from(b"config"),
            Seed::from(&seed_binding),
            Seed::from(mint_x.as_ref()),
            Seed::from(mint_y.as_ref()),
            Seed::from(bump),
        ];
        let signer = Signer::from(&config_seeds);

        MintTo{
            mint:accounts.mint_lp,
            account:accounts.user_lp_ata,
            mint_authority: accounts.config,
            amount:data.amount,
        }.invoke_signed(&[signer])?;

        Ok(())
    }
}