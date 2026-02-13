use anchor_lang::prelude::*;

// 声明所有根模块（告诉编译器去查找这些文件）
mod errors;
mod state;
mod instructions;
// use instructions::{Make,Take,Refund };
// 引入指令的 Context 结构
pub use instructions::*;

declare_id!("22222222222222222222222222222222222222222222");
#[program]

pub mod blueshift_anchor_escrow {

    use super::*;

    #[instruction(discriminator = 0)]
    pub fn make(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
        instructions::make::handler(ctx, seed, receive, amount)
    }

    #[instruction(discriminator = 1)]
    pub fn take(ctx: Context<Take>) -> Result<()> {
        instructions::take::handler(ctx)
    }

    #[instruction(discriminator = 2)]
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instructions::refund::handler(ctx)
    }
}


