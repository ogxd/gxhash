//#![feature(core_intrinsics)]
//#![feature(pointer_byte_offsets)]
#![feature(stdsimd)]
#![feature(stmt_expr_attributes)]

mod gxhash;
mod hasher;

pub use gxhash::*;
pub use hasher::*;