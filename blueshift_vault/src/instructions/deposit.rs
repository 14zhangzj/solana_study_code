use pinocchio::AccountView;
use pinocchio::error::ProgramError;
use pinocchio_system::instructions::Transfer;
use solana_address::Address;

// There is store account for deposit
pub struct DepositAccount <'info> {
    pub owner: &'info AccountView,
    pub vault: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for DepositAccount <'info> {
    type Error = ProgramError;

    fn try_from(account: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [owner, vault,_] = account else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        //Account Check
        if !owner.is_signer(){
            return Err(ProgramError::InvalidAccountOwner);  // ← 错误 1: 未签名
        }

        if !vault.owned_by(&pinocchio_system::ID){
            return Err(ProgramError::InvalidAccountOwner);  // ← 错误 2: vault 属于系统程序
        }

        if vault.lamports().ne(&0) {
            return Err(ProgramError::InvalidAccountData);  // ← 错误 3: vault 余额不为 0
        }

        let (vault_key,_) = Address::find_program_address(&[b"vault", owner.address().as_ref()], &crate::ID);
        if vault.address().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountData);  // ← 错误 4: vault 地址不匹配
        };

        Ok(Self{owner,vault})
    }
}

pub struct DepositInstruction {
    pub amount: u64,
}

impl<'info> TryFrom<&'info [u8]> for DepositInstruction {
type Error = ProgramError;
    fn try_from(data: &'info [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let amount = u64::from_le_bytes(data.try_into().unwrap());

        if amount.eq(&0){
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(Self{amount})
    }
}

pub struct Deposit <'info> {
    pub accounts: DepositAccount<'info>,
    pub instruction_data: DepositInstruction,
}

impl<'info> TryFrom<(&'info [u8], &'info [AccountView])> for Deposit<'info> {
    type Error = ProgramError;
    fn try_from((data,accounts): (&'info [u8], &'info [AccountView])) -> Result<Self, Self::Error> {
        let accounts = DepositAccount::try_from(accounts)?;
        let instruction_data = DepositInstruction::try_from(data)?;
        Ok(Self{
            accounts,
            instruction_data,
        })
    }
}

impl<'info> Deposit<'info> {
    pub const DISCRIMINATOR:&'info u8 = &0;
    pub fn process(&mut self) -> Result<(), ProgramError> {
        Transfer{
            from:self.accounts.owner,
            to:self.accounts.vault,
            lamports:self.instruction_data.amount,
        }.invoke()?;
        Ok(())
    }
}