use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {

    //首先判断数据必须大于1
    assert!(data.len() >= 1);

    //根据传入的data的第一个标号来判断进入什么逻辑
        match data[0] {
            0x00 => process_instruction_mint(program_id, accounts, &data[1..]),
            0x01 => process_instruction_transfer(program_id, accounts, &data[1..]),
            _ => unreachable!(),
        }
}

// 铸币的方法
fn process_instruction_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {

    //遍历账号
    let account_info_iter = &mut accounts.iter();
    let account_user = next_account_info(account_info_iter)?;
    let account_user_pda = next_account_info(account_info_iter)?;
    let _ = next_account_info(account_info_iter)?;
    let _ = next_account_info(account_info_iter)?;

    //检查账户权限
    assert!(account_user.is_signer);
    let account_user_pda_calc = Pubkey::find_program_address(&[&account_user.key.to_bytes()], &program_id);
    assert_eq!(account_user_pda.key, &account_user_pda_calc.0);

    //如果账户没有初始化，则新建账户
    if **account_user_pda.try_borrow_lamports().unwrap() == 0 {
        let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(8);
        let bump = account_user_pda_calc.1;
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                account_user.key,
                account_user_pda.key,
                rent_exemption,
                8,
                program_id,
            ),
            accounts,
            &[&[&account_user.key.to_bytes(), &[bump]]],
        )?;
        account_user_pda.data.borrow_mut().copy_from_slice(&u64::MIN.to_be_bytes());
    }

    //Mint
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&account_user_pda.data.borrow());

    //读取之前的金额
    let old = u64::from_be_bytes(buf);
    buf.copy_from_slice(&data);

    //读取新增的金额
    let inc = u64::from_be_bytes(buf);

    //把金额加上
    let new = old.checked_add(inc).unwrap();
    account_user_pda.data.borrow_mut().copy_from_slice(&new.to_be_bytes());
    Ok(())
}

// 转账的方法
fn process_instruction_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {

    //遍历账户
    let accounts_iter = &mut accounts.iter();
    let account_user = next_account_info(accounts_iter)?;
    let account_user_pda = next_account_info(accounts_iter)?;

    //转入账户
    let account_into = next_account_info(accounts_iter)?;
    let account_into_pda = next_account_info(accounts_iter)?;
    let _ = next_account_info(accounts_iter)?;
    let _ = next_account_info(accounts_iter)?;

    // Check accounts permissons.
    assert!(account_user.is_signer);
    let account_user_pda_calc =
        Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id);
    assert_eq!(account_user_pda.key, &account_user_pda_calc.0);
    let account_into_pda_calc =
        Pubkey::find_program_address(&[&account_into.key.to_bytes()], program_id);
    assert_eq!(account_into_pda.key, &account_into_pda_calc.0);

    if **account_into_pda.try_borrow_lamports().unwrap() == 0 {
        let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(8);
        let bump = account_into_pda_calc.1;
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                account_user.key,
                account_into_pda.key,
                rent_exemption,
                8,
                program_id,
            ),
            accounts,
            &[&[&account_into.key.to_bytes(), &[bump]]],
        )?;
        account_into_pda.data.borrow_mut().copy_from_slice(&u64::MIN.to_be_bytes());
    }

    //转账
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&account_user_pda.data.borrow());
    let old_user = u64::from_be_bytes(buf);
    buf.copy_from_slice(&account_into_pda.data.borrow());
    let old_into = u64::from_be_bytes(buf);
    buf.copy_from_slice(&data);
    let inc = u64::from_be_bytes(buf);
    let new_user = old_user.checked_sub(inc).unwrap();
    let new_into = old_into.checked_add(inc).unwrap();
    account_user_pda.data.borrow_mut().copy_from_slice(&new_user.to_be_bytes());
    account_into_pda.data.borrow_mut().copy_from_slice(&new_into.to_be_bytes());
    Ok(())
}