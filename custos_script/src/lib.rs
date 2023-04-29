pub mod vm;
pub mod bytecode;

pub mod prelude {
    pub use crate::vm::*;
    pub use crate::bytecode::*;
}
