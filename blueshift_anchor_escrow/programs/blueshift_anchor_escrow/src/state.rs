use anchor_lang::prelude::*;
#[derive(InitSpace)]
#[account(discriminator = 1)]
pub struct Escrow {
    
    //种子派生过程中的随机数，因此一个创建者可以使用相同的代币对打开多个托管账户；
    // 存储在链上，方便可以始终可以重新派生PDA
    pub seed: u64,
    
    //创建托管账户的钱包；需要用于退款和接受付款。
    pub maker: Pubkey,
    
    //交换中给出和获取两侧的SPL铸币地址
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    
    //创建者希望获得的代币B的数量。（金库的余额显示了代币A，所以不需要存储这个信息）
    pub receive: u64,
    
    //缓存的bump字节；动态派生它会消耗计算资源，因此我们讲其保存一次。
    pub bump: u8,
}