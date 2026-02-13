#![allow(unexpected_cfgs)]
pub mod program_a;
pub mod program_b;

//使用编译特性来选择程序
#[cfg(feature = "program_a")]
pub use program_a::*;

#[cfg(feature = "program_b")]
pub use program_b::*;

// 根据特性选择入口点
#[cfg(feature = "program_a")]
solana_program::entrypoint!(process_instruction);

#[cfg(feature = "program_b")]
solana_program::entrypoint!(process_instruction);