use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::sysvar::Sysvar;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {

    /*
        编写具体的功能之前，我们先思考这个功能会涉及哪些账户
        1. 用户的普通钱包账户，需要这个账户来付链上存储费
        2. 根据用户账户生成的数据账户。 如果没有的话我们会新建一个pda数据账户并存入数据。 该账户应该是可以写入的
        3. 系统账户. 只有系统账户才能创建新的账户. 该账户的权限应当是只读, 无需签名.
        4. Solana 通过 sysvar 帐户向程序公开各种集群状态数据. 在本例子中, 我们需要知道需要多少 lamport 才能使数
        据账户达成租赁豁免, 而这个数字可能由集群动态改变. 因此需要访问 sysvar rent 账户. 该账户的权限应当是
        只读, 无需签名.
    */
    let account_info_iter = &mut accounts.iter();
    let account_user = next_account_info(account_info_iter)?;
    let account_data = next_account_info(account_info_iter)?;

    //let _ 表示我们虽然读取这个数据，但是后续不会使用。那么为什么要保留这两行代码呢？是因为这两个账户都是程序
    // 必须调用的账户，如果我们写了，别人调用的时候不传是会报错的。所以虽然这个例子我们不需要，但是依然需要读取出来。
    let _ = next_account_info(account_info_iter)?;
    let _ = next_account_info(account_info_iter)?;

    //检查账户的权限,PDA账户必须是由程序为用户生成的PDA
    assert!(account_user.is_signer);

    //判断由用户种子生成的PDA地址是否和传入进来的地址是一样的
    let account_data_calc = Pubkey::find_program_address(&[&account_user.key.to_bytes()], &program_id);
    assert_eq!(account_data.key, &account_data_calc.0);

    let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(data.len());
    let bump = account_data_calc.1;

    //判断数据账户是否初始化了，如果没有的话，需要初始化，然后把数据放入其中。
    if **account_data.try_borrow_lamports().unwrap() == 0{
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                account_user.key,
                account_data.key,
                rent_exemption,
                data.len() as u64,
                program_id,
            ),
            accounts,
            &[ &[&account_user.key.to_bytes(), &[bump]] ])?;
        account_data.data.borrow_mut().copy_from_slice(data);
        return Ok(());
    }

    //如果已经存在账户，且需要存储的租金大于账户中的租金则需要进行一笔交易来补账户的保管费
    if rent_exemption > account_data.lamports(){
        solana_program::program::invoke(
            &solana_program::system_instruction::transfer(
                account_user.key,
                account_data.key,
                rent_exemption - account_data.lamports(),
            ),
            accounts
        )?;
    }

    //如果已经存在账户，且需要存储的租金小于账户中的租金则把多余的钱退给用户钱包
    if rent_exemption < account_data.lamports() {
        **account_user.lamports.borrow_mut() = account_user.lamports() + account_data.lamports() -rent_exemption;
        **account_data.lamports.borrow_mut() = rent_exemption;
    }

    //重新计算空间
    account_data.resize(data.len())?;

    //覆盖数据
    account_data.data.borrow_mut().copy_from_slice(data);
    Ok(())
}