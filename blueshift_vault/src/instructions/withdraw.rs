use pinocchio::AccountView;
use pinocchio::cpi::{Seed, Signer};
use pinocchio::error::ProgramError;
use pinocchio_system::instructions::Transfer;
use solana_address::Address;

pub struct WithdrawAccounts<'info>{
    pub owner: &'info AccountView,
    pub vault: &'info AccountView,
    pub bumps: [u8;1],
}

impl<'info> TryFrom<&'info [AccountView]> for WithdrawAccounts<'info> {
    type Error = ProgramError;
    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [owner, vault,_] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        //Account Check
        if !owner.is_signer(){
            return Err(ProgramError::InvalidAccountOwner);
        }

        if !vault.owned_by(&pinocchio_system::ID){
            return Err(ProgramError::InvalidAccountOwner);
        }

        if vault.lamports().eq(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        let (vault_key,bump) = Address::find_program_address(&[b"vault", owner.address().as_ref()], &crate::ID);
        if vault.address().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountOwner);
        };

        Ok(Self{owner,vault,bumps:[bump]})
    }
}

pub struct Withdraw<'info>{
    pub accounts: WithdrawAccounts<'info>,
}

impl<'info> TryFrom<&'info [AccountView]> for Withdraw<'info>{
    type Error = ProgramError;
    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let accounts = WithdrawAccounts::try_from(accounts)?;
        Ok(Self{accounts})
    }
}

impl<'info> Withdraw<'info>{
    pub const DISCRIMINATOR:&'info u8 = &1;

    pub fn process(&mut self) -> Result<(), ProgramError> {
        let seeds = [
            Seed::from(b"vault"),
            Seed::from(self.accounts.owner.address().as_ref()),
            Seed::from(&self.accounts.bumps)
        ];
        let signers = [Signer::from(&seeds)];

        Transfer{
            from:self.accounts.vault,
            to:self.accounts.owner,
            lamports:self.accounts.vault.lamports()
        }.invoke_signed(&signers)?;
        Ok(())
    }
}