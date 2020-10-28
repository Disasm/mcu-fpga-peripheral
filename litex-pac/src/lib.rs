#![no_std]

use cortex_m as arch;

#[path = "register_shim.rs"]
pub mod register;
pub use crate::register::{RORegister, UnsafeRORegister};
pub use crate::register::{WORegister, UnsafeWORegister};
pub use crate::register::{RWRegister, UnsafeRWRegister};

mod soc;
pub use soc::*;
