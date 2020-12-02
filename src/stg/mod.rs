pub mod ast;
pub mod machine;
pub mod transform;
pub mod heap;

pub use machine::StgMachine;

#[cfg(test)]
mod tests;
