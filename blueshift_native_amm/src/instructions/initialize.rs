use std::mem::MaybeUninit;
use pinocchio::{AccountView, ProgramResult};
use pinocchio::cpi::{Seed, Signer};
use pinocchio::error::ProgramError;
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::Sysvar;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::InitializeMint2;
use pinocchio_token::state::Mint;
use crate::Config;

pub struct InitializeAccounts<'info>{
    pub initializer: &'info AccountView,
    pub mint_lp: &'info AccountView,
    pub config: &'info AccountView,
    pub token_program: &'info AccountView,
    pub system_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for InitializeAccounts<'info>{
    type Error = ProgramError;
    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error>{
        let [initializer, mint_lp, config, token_program, system_program] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self{
            initializer,
            mint_lp,
            config,
            token_program,
            system_program,
        })

    }
}

#[repr(C,packed)]
pub struct InitializeInstructionData {
    pub seed: u64,
    pub fee: u16,
    pub mint_x: [u8; 32],
    pub mint_y: [u8; 32],
    pub config_bump: [u8;1],
    pub lp_bump: [u8;1],
    pub authority: [u8;32]
}
impl TryFrom<&[u8]> for InitializeInstructionData {
    type Error = ProgramError;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error>{
        const INITIALIZE_DATA_LEN_WITH_AUTHORITY: usize = size_of::<InitializeInstructionData>();
        const INITIALIZE_DATA_LEN: usize =
            INITIALIZE_DATA_LEN_WITH_AUTHORITY - size_of::<[u8;32]>();

        match data.len() {
            INITIALIZE_DATA_LEN_WITH_AUTHORITY => {
                Ok(unsafe{ (data.as_ptr() as *const Self).read_unaligned() })
            }
            INITIALIZE_DATA_LEN => {
                let mut raw: MaybeUninit<[u8;INITIALIZE_DATA_LEN_WITH_AUTHORITY]> = MaybeUninit::uninit();
                let raw_ptr = raw.as_mut_ptr() as *mut u8;
                unsafe {
                    core::ptr::copy_nonoverlapping(data.as_ptr(), raw_ptr, INITIALIZE_DATA_LEN);
                    core::ptr::write_bytes(raw_ptr.add(INITIALIZE_DATA_LEN),0,32);
                    Ok((raw.as_ptr() as *const Self).read_unaligned())
                }
            }

            _ => Err(ProgramError::InvalidInstructionData),
        }

    }
}

pub struct Initialize<'info>{
    pub initialize_accounts : InitializeAccounts<'info>,
    pub initialize_instruction_data: InitializeInstructionData
}

impl<'info> TryFrom<(&'info [u8], &'info [AccountView])> for Initialize<'info> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'info [u8], &'info [AccountView])) -> Result<Self, Self::Error> {
        let initialize_accounts = InitializeAccounts::try_from(accounts)?;
        let initialize_instruction_data = InitializeInstructionData::try_from(data)?;

        Ok(Self{
            initialize_accounts,
            initialize_instruction_data
        })
    }
}

impl<'info> Initialize<'info> {
    pub const DISCRIMINATOR: &'info u8 = &0;

    pub fn process(&self) -> ProgramResult{

        //使用CreateAccount 和种子创建Config账户
        let instruction_data = &self.initialize_instruction_data;
        let accounts = &self.initialize_accounts;
        let rent = Rent::get()?;

        let config_lamports = rent.try_minimum_balance(Config::LEN)?;
        let seed_binding = self.initialize_instruction_data.seed.to_le_bytes();
        let config_seeds = [
            Seed::from(b"config"),
            Seed::from(&seed_binding),
            Seed::from(&instruction_data.mint_x),
            Seed::from(&instruction_data.mint_y),
            Seed::from(&instruction_data.config_bump),
        ];

        let config_signer = Signer::from(&config_seeds);
        CreateAccount{
            from:accounts.initializer,
            to:accounts.config,
            lamports:config_lamports,
            space:Config::LEN as u64,
            owner:&crate::ID,
        }.invoke_signed(&[config_signer])?;

        //初始化Config数据

        let config_account = unsafe { Config::load_mut_unchecked(accounts.config)? };
        config_account.set_inner(
            instruction_data.seed,
            instruction_data.authority.into(), // 将 [u8;32] 转为 Pubkey
            instruction_data.mint_x.into(),
            instruction_data.mint_y.into(),
            instruction_data.fee,
            instruction_data.config_bump,
        )?;

        let mint_lp_seeds = [
            Seed::from(b"mint_lp"),
            Seed::from(accounts.config.address().as_array()),
            Seed::from(&instruction_data.lp_bump),
        ];

        //创建Mint LP账户
        let mint_space = size_of::<Mint>();
        let mint_lamports = rent.try_minimum_balance(mint_space)?;

        CreateAccount{
            from:accounts.initializer,
            to:accounts.mint_lp,
            lamports:mint_lamports,
            space:mint_space as u64,
            owner:&pinocchio_token::ID,
        }.invoke_signed(&[Signer::from(&mint_lp_seeds)])?; //Mint的所有者是Token Program

        //初始化Mint LP
        InitializeMint2{
            mint:accounts.mint_lp,
            decimals:6,
            mint_authority:accounts.config.address(),
            freeze_authority:None,
        }.invoke()?;

        Ok(())
    }
}