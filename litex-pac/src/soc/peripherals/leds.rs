#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! LEDS

use crate::{RWRegister};
use core::marker::PhantomData;

pub mod OUT {
    /// The center Red LED #1 on the iCEBreaker head.
    pub mod hledr1 {
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
    
    }
    /// Green LED #2 on the iCEBreaker head.
    pub mod hledg2 {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
    
        /// Mask (1 bit: 0x1 << 1)
        pub const mask: u32 = 0x1 << offset;
    
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    
    }
    /// Green LED #3 on the iCEBreaker head.
    pub mod hledg3 {
        /// Offset (2 bits)
        pub const offset: u32 = 2;
    
        /// Mask (1 bit: 0x1 << 2)
        pub const mask: u32 = 0x1 << offset;
    
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    
    }
    /// Green LED #4 on the iCEBreaker head.
    pub mod hledg4 {
        /// Offset (3 bits)
        pub const offset: u32 = 3;
    
        /// Mask (1 bit: 0x1 << 3)
        pub const mask: u32 = 0x1 << offset;
    
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    
    }
    /// Green LED #5 on the iCEBreaker head.
    pub mod hledg5 {
        /// Offset (4 bits)
        pub const offset: u32 = 4;
    
        /// Mask (1 bit: 0x1 << 4)
        pub const mask: u32 = 0x1 << offset;
    
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    
    }}

pub struct RegisterBlock {
    pub OUT: RWRegister<u32>,
}

pub struct ResetValues {
    pub OUT: u32,
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
