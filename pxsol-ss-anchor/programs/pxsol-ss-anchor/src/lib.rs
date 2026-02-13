// ==================== Anchor 框架导入 ====================
// use anchor_lang::prelude::*; 相当于原生实现中的多个导入：
// 对应 lib_before.rs: use solana_program::entrypoint::ProgramResult;
// 对应 lib_before.rs: use solana_program::pubkey::Pubkey;
// 对应 lib_before.rs: use solana_program::account_info::{next_account_info, AccountInfo};
// 对应 lib_before.rs: use solana_program::sysvar::Sysvar;
// Anchor 将所有常用的模块整合到了 prelude 中，简化了导入
use anchor_lang::prelude::*;

// ==================== 程序 ID 声明 ====================
// 声明当前程序的 Program ID
// 在原生实现中，这个 ID 是在部署时由 Solana 链分配的，或者通过 BPFLoader 指定
// Anchor 在编译时会自动检查这个 ID 是否匹配
declare_id!("7ZTR83ymve1Fj8x8KNdoFVWg7Wk9JXGZuYFqx7Y8wivh");

// ==================== 程序模块定义 ====================
// #[program] 宏是 Anchor 的核心特性，它自动生成：
// 1. 程序的 entrypoint（入口点）
// 2. 指令分发逻辑（根据指令数据调用对应的函数）
// 3. 序列化/反序列化逻辑
//
// 对应 lib_before.rs 中的整个 process_instruction 函数（第6-79行）
// 在原生实现中，你需要手动解析账户、验证权限、处理指令分发
// Anchor 的 #[program] 宏自动处理了这些样板代码
#[program]
pub mod pxsol_ss_anchor {
    use super::*;

    // ==================== 初始化指令 ====================
    // initialize 函数对应 lib_before.rs 中创建新数据账户的逻辑（第41-54行）
    // 在原生实现中，创建账户和存储数据的逻辑都混在 process_instruction 中
    // Anchor 将初始化和更新分离成两个独立的指令函数
    //
    // Context<Init> 包含了所有需要的账户信息，Anchor 会自动验证账户
    // 返回类型 Result<()> 是 Anchor 的结果类型，相当于 ProgramResult
    pub fn initialize(ctx: Context<Init>) -> Result<()> {
        // ==================== 获取账户引用 ====================
        // &mut 表示可变引用，让我们可以修改账户数据
        // ctx.accounts 是一个自动生成的结构体，包含所有在 Init 结构体中定义的账户
        // 对应 lib_before.rs:22-23 中的手动账户获取：
        //     let account_user = next_account_info(account_info_iter)?;
        //     let account_data = next_account_info(account_info_iter)?;
        let account_user = &mut ctx.accounts.user;
        let account_user_pda = &mut ctx.accounts.user_pda;

        // ==================== 设置授权和 bump seed ====================
        // 将用户的公钥存储到 PDA 账户中，用于后续验证
        // 对应 lib_before.rs:34 中的 PDA 验证逻辑
        account_user_pda.auth = account_user.key();

        // ctx.bumps.user_pda 是 Anchor 自动计算并传入的 bump seed
        // 对应 lib_before.rs:38 中的手动计算：
        //     let bump = account_data_calc.1;
        account_user_pda.bump = ctx.bumps.user_pda;

        // 初始化数据为空向量
        // 对应 lib_before.rs:52 中的数据存储：
        //     account_data.data.borrow_mut().copy_from_slice(data);
        account_user_pda.data = Vec::new();

        // 返回成功，相当于 Ok(())
        Ok(())
    }

    // ==================== 更新指令 ====================
    // update 函数对应 lib_before.rs 中更新已有账户数据的逻辑（第56-79行）
    //
    // 除了 Context，还可以接收自定义参数（如这里的 data: Vec<u8>）
    // Anchor 会自动处理参数的序列化/反序列化
    // 对应 lib_before.rs:9 中的 data: &[u8] 参数
    pub fn update(ctx: Context<Update>, data: Vec<u8>) -> Result<()> {
        // 获取账户引用
        // 对应 lib_before.rs:22-23
        let account_user = &ctx.accounts.user;
        let account_user_pda = &mut ctx.accounts.user_pda;

        // ==================== 更新数据 ====================
        // 直接赋值即可，Anchor 自动处理序列化
        // 这行代码会触发账户的重新分配（resize），因为我们在 Update 结构体中指定了 realloc
        // 对应 lib_before.rs:75-78：
        //     account_data.resize(data.len())?;
        //     account_data.data.borrow_mut().copy_from_slice(data);
        //
        // 注意：Anchor 的 realloc 约束会自动处理租金调整：
        // - 扩大账户时：从 payer 转移 lamports 到 PDA 以保持租金豁免
        // - 缩小账户时：自动将多余的 lamports 退还给 payer
        // 因此不需要手动处理租金退款逻辑
        account_user_pda.data = data;

        // ==================== 手动处理租金退款 ====================
        // 注意：虽然 Anchor 的 realloc 约束会在账户缩小时自动处理租金退款，
        // 但下面这段代码演示了如何手动处理租金退款逻辑。
        //
        // 对应 lib_before.rs:68-73 的退款逻辑：
        //     let hold = **account_data.lamports.borrow();
        //     if hold > rent_exempt {
        //         let refund = hold - rent_exempt;
        //         **account_data.lamports.borrow_mut() = rent_exempt;
        //         **account_user.lamports.borrow_mut() += refund;
        //     }
        //
        // 实际上，这段代码是冗余的，因为 Anchor 的 realloc 约束已经处理了这个逻辑。
        // 保留这段代码仅用于学习目的，展示如何在 Anchor 中手动操作账户余额。

        // 将 Account 对象转换为底层的 AccountInfo，以便访问底层数据
        // to_account_info() 返回一个包装了账户信息的引用
        let account_user_pda_info = account_user_pda.to_account_info();

        // 获取当前租金豁免所需的最低 lamports 余额
        // Rent::get()? 从 sysvar 租金账户获取当前租金配置
        // minimum_balance(len) 计算指定数据长度的账户所需的租金豁免金额
        // 对应 lib_before.rs:57：let rent_exempt = Rent::get()?.minimum_balance(account_data.data_len());
        let rent_exemption = Rent::get()?.minimum_balance(account_user_pda_info.data_len());

        // 获取 PDA 账户当前持有的 lamports 数量
        // ** 双重解引用：&RefCell<u64> -> &u64 -> u64
        // .lamports.borrow() 返回 Ref<u64>，** 解引用得到实际值
        // 对应 lib_before.rs:68：let hold = **account_data.lamports.borrow();
        let hold = **account_user_pda_info.lamports.borrow();

        // 如果账户持有的 lamports 超过租金豁免所需金额，说明账户被缩小了
        // 需要将多余的 lamports 退还给支付者
        if hold > rent_exemption {
            // 计算需要退还的金额
            // saturating_sub 是饱和减法，防止下溢（虽然这里不会发生，因为已检查 hold > rent_exemption）
            // 对应 lib_before.rs:69：let refund = hold - rent_exempt;
            let refund = hold.saturating_sub(rent_exemption);

            // 将 PDA 账户的 lamports 减少到租金豁免所需的最小金额
            // ** 左侧是解引用，允许我们修改 RefCell 中的值
            // 对应 lib_before.rs:70：**account_data.lamports.borrow_mut() = rent_exempt;
            **account_user_pda_info.lamports.borrow_mut() = rent_exemption;

            // 将退还的 lamports 转账回用户账户
            // checked_add 是安全的加法，会检查溢出并返回 Option
            // unwrap() 安全，因为这里不可能溢出
            // 对应 lib_before.rs:71：**account_user.lamports.borrow_mut() += refund;
            **account_user.lamports.borrow_mut() = account_user.lamports().checked_add(refund).unwrap();
        }

        Ok(())
    }
}

// ==================== Init 账户验证结构体 ====================
// #[derive(Accounts)] 宏自动生成账户验证逻辑
// 这个结构体定义了 initialize 指令需要哪些账户，以及如何验证它们
//
// 对应 lib_before.rs:12-20 中的注释和账户获取逻辑：
//     "1. 用户的普通钱包账户，需要这个账户来付链上存储费"
//     "2. 根据用户账户生成的数据账户..."
//     "3. 系统账户..."
//     "4. sysvar rent 账户..."
// Anchor 通过 Accounts 宏自动处理这些账户的验证
#[derive(Accounts)]
pub struct Init<'info> {
    // ==================== 用户签名账户 ====================
    // #[account(mut)] 表示这个账户的状态（lamports）可能会被修改
    // Signer<'info> 表示这个账户必须签名，但不一定是可写入的
    //
    // 对应 lib_before.rs:22：
    //     let account_user = next_account_info(account_info_iter)?;
    // 对应 lib_before.rs:31 的验证：
    //     assert!(account_user.is_signer);
    // Anchor 自动验证这个账户是否签名，无需手动检查
    #[account(mut)]
    pub user: Signer<'info>,

    // ==================== 用户 PDA 账户 ====================
    // 这个宏定义包含了多个验证和创建指令：
    //
    // init: 创建新的 PDA 账户
    //       对应 lib_before.rs:42-51 的 invoke_signed 调用
    //
    // payer = user: 指定由 user 账户支付创建费用
    //       对应 lib_before.rs:43 中的 account_user.key（作为支付方）
    //
    // seeds = [SEED, user.key().as_ref()], 定义 PDA 的种子
    //       对应 lib_before.rs:34 中的 &[&account_user.key.to_bytes()]
    //
    // bump: 让 Anchor 自动计算 bump seed
    //       对应 lib_before.rs:38：let bump = account_data_calc.1;
    //
    // space = Data::space_for(0): 分配的账户空间大小
    //       对应 lib_before.rs:47：data.len() as u64
    //
    // Account<'info, Data>: 将账户反序列化为 Data 结构体
    //       在原生实现中需要手动解析数据，Anchor 自动处理
    #[account(
        init,
        payer = user,
        seeds = [SEED, user.key().as_ref()],
        bump,
        space = Data::space_for(0)
    )]
    pub user_pda: Account<'info, Data>,

    // ==================== 系统程序 ====================
    // Program<'info, System> 表示必须包含系统程序账户
    // 创建账户需要调用系统程序，所以必须传入
    //
    // 对应 lib_before.rs:27：
    //     let _ = next_account_info(account_info_iter)?;
    // 那个 _ 就是为了读取（但不使用）系统程序账户
    // Anchor 自动验证这个账户是否是 System Program
    pub system_program: Program<'info, System>,
}

// ==================== Data 账户数据结构 ====================
// #[account] 宏为这个结构体实现以下功能：
// 1. 自动实现 Borsh 序列化/反序列化（存储到链上需要）
// 2. 添加账户头部（discriminator，8字节）
// 3. 实现必要的 trait 以便与 Anchor 框架集成
//
// 对应 lib_before.rs:52 中手动存储数据：
//     account_data.data.borrow_mut().copy_from_slice(data);
// 在原生实现中，数据是原始字节，需要手动解析
// Anchor 使用结构体让数据有类型，自动处理序列化
#[account]
pub struct Data {
    // 存储授权用户的公钥（用于后续验证）
    // 在原生实现中，没有这个字段，因为每次调用都验证 PDA
    // Anchor 版本在初始化时记录授权用户，update 时验证
    pub auth: Pubkey,

    // 存储 PDA 的 bump seed
    // 对应 lib_before.rs:38：let bump = account_data_calc.1;
    // 在原生实现中，bump 是在运行时计算或从签名派生中获取
    // Anchor 版本将 bump 存储在账户中，方便后续验证
    pub bump: u8,

    // 存储实际的数据（字节数组）
    // 对应 lib_before.rs:52 中存储的 data
    pub data: Vec<u8>
}

// ==================== Data 结构体的辅助函数 ====================
impl Data {
    // 计算账户需要分配的空间大小
    //
    // Solana 账户数据布局：
    // 8 字节: discriminator（Anchor 自动添加，用于识别账户类型）
    // 32 字节: Pubkey (auth 字段)
    // 1 字节: u8 (bump 字段)
    // 4 字节: Vec 的长度前缀
    // data_len: 实际数据长度
    //
    // 对应 lib_before.rs:47 中的 data.len() as u64
    // 原生实现只需要传入数据长度，Anchor 需要计算完整的账户空间
    pub fn space_for(data_len: usize) -> usize {
        8 + 32 + 1 + 4 + data_len
    }
}

// 定义 Update 指令使用的 seed 常量
// 对应 lib_before.rs:34 中使用用户公钥作为种子：
//     &[&account_user.key.to_bytes()]
// 这里使用固定的 "data" 种子加上用户公钥
const SEED: &[u8] = b"data";

// ==================== Update 账户验证结构体 ====================
// 定义 update 指令需要的账户
#[derive(Accounts)]
// #[instruction] 宏允许在结构体中使用指令参数
// 这里我们需要知道 new_data 的长度来计算重新分配的空间大小
#[instruction(new_data: Vec<u8>)]
pub struct Update<'info> {
    // ==================== 用户签名账户 ====================
    // 对应 lib_before.rs:22：let account_user = next_account_info(account_info_iter)?;
    #[account(mut)]
    pub user: Signer<'info>,

    // ==================== 已存在的用户 PDA 账户 ====================
    // 注意这里没有 init，因为账户已经存在
    #[account(
        mut,  // 账户数据可修改

        // 验证 PDA 地址是否正确
        // 对应 lib_before.rs:34-35：
        //     let account_data_calc = Pubkey::find_program_address(&[&account_user.key.to_bytes()], &program_id);
        //     assert_eq!(account_data.key, &account_data_calc.0);
        seeds = [SEED, user.key().as_ref()],
        bump = user_pda.bump,  // 使用存储的 bump 来验证

        // 重新分配账户空间（如果需要）
        // 对应 lib_before.rs:56-66 的补足租金逻辑和 75 的 resize 调用
        // Anchor 自动处理：
        // 1. 计算新空间大小
        // 2. 检查是否需要补足租金（lib_before.rs:56-66）
        // 3. 调用系统程序进行 transfer 和 realloc
        realloc = Data::space_for(new_data.len()),
        realloc::payer = user,  // 由用户支付额外的租金
        realloc::zero = false,  // 不对新增空间进行零填充

        // 自定义约束验证：只有授权用户才能更新
        // 对应 lib_before.rs:34 的 PDA 验证逻辑
        // 这里增加了额外的授权检查，确保只有初始化时的用户才能更新
        constraint = user_pda.auth == user.key() @ PxsolError::Unauthorized,
    )]
    pub user_pda: Account<'info, Data>,

    // 系统程序（重新分配账户空间时需要）
    // 对应 lib_before.rs:27-28 读取系统程序账户
    pub system_program: Program<'info, System>,
}

// ==================== 自定义错误类型 ====================
// #[error_code] 宏自动生成错误代码和消息
//
// 对应 lib_before.rs 中使用的 assert! 和 assert_eq!
// 在原生实现中，如果断言失败，程序会 panic
// Anchor 使用自定义错误提供了更好的错误处理和用户体验
#[error_code]
pub enum PxsolError {
    // 定义错误码和错误消息
    // 当 constraint 验证失败时，返回这个错误
    #[msg("Account is not authorized.")]
    Unauthorized
}
