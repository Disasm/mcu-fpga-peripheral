#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! CTRL

use crate::{RORegister, RWRegister};
use core::marker::PhantomData;

/// Write a ``1`` to this register to reset the SoC.
pub mod RESET {
    pub mod reset {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
    
        /// Mask (1 bit: 0x1 << 0)
        pub const mask: u32 = 0x1 << offset;
    
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    
    }}

/// Use this register as a scratch space to verify that software read/write accesses
/// to the Wishbone/CSR bus are working correctly. The initial reset value of
/// 0x1234578 can be used to verify endianness.
pub mod SCRATCH {
    pub mod scratch {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
    
        /// Mask (32 bit: 0xffffffff << 0)
        pub const mask: u32 = 0xffffffff << offset;
    
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    
    }}

/// Total number of Wishbone bus errors (timeouts) since start.
pub mod BUS_ERRORS {
    pub mod bus_errors {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
    
        /// Mask (32 bit: 0xffffffff << 0)
        pub const mask: u32 = 0xffffffff << offset;
    
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    
    }}

pub struct RegisterBlock {
    /// Write a ``1`` to this register to reset the SoC.
    pub RESET: RWRegister<u32>,

    /// Use this register as a scratch space to verify that software read/write accesses
    /// to the Wishbone/CSR bus are working correctly. The initial reset value of
    /// 0x1234578 can be used to verify endianness.
    pub SCRATCH: RWRegister<u32>,

    /// Total number of Wishbone bus errors (timeouts) since start.
    pub BUS_ERRORS: RORegister<u32>,
}

pub struct ResetValues {
    pub RESET: u32,
    pub SCRATCH: u32,
    pub BUS_ERRORS: u32,
}

pub struct Instance {
    pub(crate) addr: u32,
    pub(crate) _marker: PhantomData<*const RegisterBlock>,
}

impl ::core::ops::Deref for Instance {
    type Target = RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*(self.addr as *const _) }
    }
}
