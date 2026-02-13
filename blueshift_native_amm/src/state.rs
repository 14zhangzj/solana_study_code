// 引入 core::mem::size_of 用于在编译时获取类型的大小
use core::mem::size_of;
// 引入 pinocchio 框架的核心类型：AccountView（账户视图）、Address（地址类型）
use pinocchio::{AccountView, Address};
// 引入 Ref 类型，用于创建对账户数据的引用包装器
use pinocchio::account::{Ref, RefMut};
// 引入 Seed 类型，用于跨程序调用的 PDA 种子签名
use pinocchio::cpi::Seed;
// 引入 ProgramError 枚举，定义 Solana 程序的标准错误类型
use pinocchio::error::ProgramError;

// 使用 C 语言内存布局，确保结构体字段按声明顺序在内存中连续排列
// 这对于将原始字节数组直接解释为结构体至关重要
#[repr(C)]
pub struct Config {
    // AMM 状态字段，存储 AmmState 枚举的 u8 值（0-3）
    state: u8,
    // 种子值，用于 PDA（Program Derived Address）派生，存储为 8 字节小端序
    seed: [u8; 8],
    // 拥有此配置账户权限的地址（32 字节）
    authority: Address,
    // 第一个代币的 mint 账户地址（32 字节）
    mint_x: Address,
    // 第二个代币的 mint 账户地址（32 字节）
    mint_y: Address,
    // 手续费比率，存储为 2 字节数组（可能是 bps 或其他费率表示）
    fee: [u8;2],
    // 配置账户的 PDA bump seed，用于验证地址派生
    config_bump: [u8;1],
}

// 使用 u8 内存布局的枚举，确保每个枚举值只占 1 字节
// 这样可以直接与 Config 结构体中的 state: u8 字段对应
#[repr(u8)]
pub enum AmmState {
    // 未初始化状态，账户刚创建但未设置
    Uninitialized = 0u8,
    // 已初始化状态，AMM 正常运行
    Initialized = 1u8,
    // 已禁用状态，暂停所有操作
    Disabled = 2u8,
    // 仅提现状态，允许流动性移除但不允许添加或交易
    WithdrawOnly = 3u8,
}

// Config 结构体的实现块，定义相关方法
impl Config {
    // 常量 LEN：在编译时计算 Config 结构体的字节大小
    // 用于验证账户数据长度是否正确
    pub const LEN: usize = size_of::<Config>();

    // 安全地加载 Config 账户数据，返回引用包装器 Ref<Self>
    // 使用内联提示（inline）避免函数调用开销
    #[inline(always)]
    pub fn load(account_info: &AccountView) -> Result<Ref<Self>, ProgramError> {
        // 验证账户数据长度是否与 Config 结构体大小匹配
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        // 在 unsafe 块中验证账户所有者是否为当前程序
        if !account_info.owned_by(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        // 创建对账户数据的引用映射，零拷贝转换为 Config 引用
        // try_borrow() 获取可变借用检查，然后使用 Ref::map 将字节切片映射为 Config
        Ok(Ref::map(account_info.try_borrow()?, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    // 不安全的加载方法，直接获取 &Self 引用而不经过借用检查
    // 性能更高但需要调用者确保没有可变别名冲突
    #[inline(always)]
    pub unsafe fn load_unchecked(account_info: &AccountView) -> Result<&Self, ProgramError > {
        // 验证账户数据长度
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        // 验证账户所有者（不需要 unsafe 块，因为这不是借用操作）
        if account_info.owner().ne(&crate::ID){
            return Err(ProgramError::InvalidAccountOwner);
        }
        // 直接获取不可变借用而不进行借用检查
        // 将字节数组转换为 Config 引用
        Ok(Self::from_bytes_unchecked(
            account_info.borrow_unchecked(),
        ))
    }

    // 核心：将不可变字节切片转换为 Config 引用（零拷贝）
    // 这就是"为什么这样读取数据"的关键
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked( bytes: &[u8]) -> &Self {
        // 获取字节切片的指针，将其重新解释为 Config 类型的常量指针，然后解引用
        // &* 创建引用，避免实际解引用的开销
        &*(bytes.as_ptr() as *const Config)
    }

    // 将可变字节切片转换为可变 Config 引用（零拷贝）
    // 允许直接修改账户数据
    #[inline(always)]
    pub unsafe fn from_bytes_unchecked_mut( bytes: &mut [u8]) -> &mut Self {
        // 同上，但使用可变指针
        &mut *(bytes.as_mut_ptr() as *mut Config)
    }

    // Getter：返回 AMM 状态值
    #[inline(always)]
    pub fn state(&self) -> u8 {
        self.state
    }

    // Getter：返回 seed 的 u64 表示（从小端字节数组转换）
    #[inline(always)]
    pub fn seed(&self) -> u64 {
        u64::from_le_bytes(self.seed)
    }

    // Getter：返回权限管理员的地址引用
    #[inline(always)]
    pub fn authority(&self) -> &Address {
        &self.authority
    }

    // Getter：返回第一个代币的 mint 地址
    #[inline(always)]
    pub fn mint_x(&self) -> &Address {
        &self.mint_x
    }

    // Getter：返回第二个代币的 mint 地址
    #[inline(always)]
    pub fn mint_y(&self) -> &Address {
        &self.mint_y
    }

    // Getter：返回手续费字节数组引用
    #[inline(always)]
    pub fn fee(&self) -> &[u8;2] {
        &self.fee
    }

    // Getter：返回配置账户的 bump seed
    #[inline(always)]
    pub fn config_bump(&self) -> &[u8;1] {
        &self.config_bump
    }

    #[inline(always)]
    pub fn load_mut(account_info: &AccountView) -> Result<RefMut<Self>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if !account_info.owned_by(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(RefMut::map(account_info.try_borrow_mut()?, |data| unsafe {
            Self::from_bytes_unchecked_mut(data)
        }))
    }

    #[inline(always)]
    pub unsafe fn load_mut_unchecked(account_info: &AccountView) -> Result<&mut Self, ProgramError > {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(unsafe {Self::from_bytes_unchecked_mut(account_info.borrow_unchecked_mut())})
    }

    #[inline(always)]
    pub fn set_state(&mut self, state: u8) -> Result<(), ProgramError> {
        if state.ge(&(AmmState::WithdrawOnly as u8)){
            return Err(ProgramError::InvalidAccountData);
        }
        self.state = state as u8;
        Ok(())
    }

    #[inline(always)]
    pub fn set_fee(&mut self, fee: u16) -> Result<(), ProgramError> {
        if fee.ge(&10_000){
            return Err(ProgramError::InvalidAccountData);
        };
        self.fee = fee.to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed.to_le_bytes();
    }

    #[inline(always)]
    pub fn set_authority(&mut self, authority: Address) {
        self.authority = authority;
    }

    #[inline(always)]
    pub fn set_mint_x(&mut self, mint_x: Address) {
        self.mint_x = mint_x;
    }

    #[inline(always)]
    pub fn set_mint_y(&mut self, mint_y: Address) {
        self.mint_y = mint_y;
    }

    #[inline(always)]
    pub fn set_config_bump(&mut self, config_bump: [u8; 1]) {
        self.config_bump = config_bump;
    }

    #[inline(always)]
    pub fn set_inner(
        &mut self,
        seed: u64,
        authority: Address,
        mint_x: Address,
        mint_y: Address,
        fee: u16,
        config_bump: [u8;1],
    ) -> Result<(), ProgramError> {
        self.set_state(AmmState::Initialized as u8)?;
        self.set_seed(seed);
        self.set_authority(authority);
        self.set_mint_x(mint_x);
        self.set_mint_y(mint_y);
        self.set_fee(fee);
        self.set_config_bump(config_bump);
        Ok(())
    }

    #[inline(always)]
    pub fn has_authority(&self) -> Option<Address> {
        let auth = unsafe {core::ptr::addr_of!(self.authority).read_unaligned()};

        if auth == Address::default(){
            None
        }else {
            Some(auth)
        }
    }
}