//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::ops::{Range, RangeInclusive};

use sel4::{sel4_cfg, UnknownSyscall, UnknownSyscallInIpcBuffer, UserContext, Word};

pub use sel4_supervising_types::{
    MemoryAccessData as VmFaultData, MemoryAccessWidth as VmFaultWidth,
};

mod arch;

pub use arch::*;

macro_rules! inner_decls {
    ($ty:ty) => {
        fn inner(&self) -> &$ty;

        fn inner_mut(&mut self) -> &mut $ty;
    };
}

use inner_decls;

macro_rules! self_impl {
    ($trt:ty, $ty:ty) => {
        impl $trt for $ty {
            fn inner(&self) -> &Self {
                self
            }

            fn inner_mut(&mut self) -> &mut Self {
                self
            }
        }
    };
}

use self_impl;

self_impl!(UserContextExt, UserContext);

pub trait UserContextExt {
    inner_decls!(UserContext);

    #[sel4_cfg(ARCH_AARCH64)]
    fn c_param(&self, ix: usize) -> &Word {
        match ix {
            0 => self.inner().x0(),
            1 => self.inner().x1(),
            2 => self.inner().x2(),
            3 => self.inner().x3(),
            4 => self.inner().x4(),
            5 => self.inner().x5(),
            6 => self.inner().x6(),
            7 => self.inner().x7(),
            _ => panic!(),
        }
    }

    #[sel4_cfg(ARCH_AARCH64)]
    fn c_param_mut(&mut self, ix: usize) -> &mut Word {
        match ix {
            0 => self.inner_mut().x0_mut(),
            1 => self.inner_mut().x1_mut(),
            2 => self.inner_mut().x2_mut(),
            3 => self.inner_mut().x3_mut(),
            4 => self.inner_mut().x4_mut(),
            5 => self.inner_mut().x5_mut(),
            6 => self.inner_mut().x6_mut(),
            7 => self.inner_mut().x7_mut(),
            _ => panic!(),
        }
    }

    #[sel4_cfg(ARCH_AARCH32)]
    fn c_param(&self, ix: usize) -> &Word {
        match ix {
            0 => self.inner().r0(),
            1 => self.inner().r1(),
            2 => self.inner().r2(),
            3 => self.inner().r3(),
            4 => self.inner().r4(),
            5 => self.inner().r5(),
            6 => self.inner().r6(),
            7 => self.inner().r7(),
            _ => panic!(),
        }
    }

    #[sel4_cfg(ARCH_AARCH32)]
    fn c_param_mut(&mut self, ix: usize) -> &mut Word {
        match ix {
            0 => self.inner_mut().r0_mut(),
            1 => self.inner_mut().r1_mut(),
            2 => self.inner_mut().r2_mut(),
            3 => self.inner_mut().r3_mut(),
            4 => self.inner_mut().r4_mut(),
            5 => self.inner_mut().r5_mut(),
            6 => self.inner_mut().r6_mut(),
            7 => self.inner_mut().r7_mut(),
            _ => panic!(),
        }
    }

    #[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))]
    fn c_param(&self, ix: usize) -> &Word {
        match ix {
            0 => self.inner().a0(),
            1 => self.inner().a1(),
            2 => self.inner().a2(),
            3 => self.inner().a3(),
            4 => self.inner().a4(),
            5 => self.inner().a5(),
            6 => self.inner().a6(),
            7 => self.inner().a7(),
            _ => panic!(),
        }
    }

    #[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))]
    fn c_param_mut(&mut self, ix: usize) -> &mut Word {
        match ix {
            0 => self.inner_mut().a0_mut(),
            1 => self.inner_mut().a1_mut(),
            2 => self.inner_mut().a2_mut(),
            3 => self.inner_mut().a3_mut(),
            4 => self.inner_mut().a4_mut(),
            5 => self.inner_mut().a5_mut(),
            6 => self.inner_mut().a6_mut(),
            7 => self.inner_mut().a7_mut(),
            _ => panic!(),
        }
    }

    #[sel4_cfg(ARCH_X86_64)]
    fn c_param(&self, ix: usize) -> &Word {
        match ix {
            0 => self.inner().rdi(),
            1 => self.inner().rsi(),
            2 => self.inner().rdx(),
            3 => self.inner().rcx(),
            4 => self.inner().r8(),
            5 => self.inner().r9(),
            _ => panic!(),
        }
    }

    #[sel4_cfg(ARCH_X86_64)]
    fn c_param_mut(&mut self, ix: usize) -> &mut Word {
        match ix {
            0 => self.inner_mut().rdi_mut(),
            1 => self.inner_mut().rsi_mut(),
            2 => self.inner_mut().rdx_mut(),
            3 => self.inner_mut().rcx_mut(),
            4 => self.inner_mut().r8_mut(),
            5 => self.inner_mut().r9_mut(),
            _ => panic!(),
        }
    }

    #[sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32, ARCH_RISCV64, ARCH_RISCV32))]
    fn generic_pc(&self) -> &Word {
        self.inner().pc()
    }

    #[sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32, ARCH_RISCV64, ARCH_RISCV32))]
    fn generic_pc_mut(&mut self) -> &mut Word {
        self.inner_mut().pc_mut()
    }

    #[sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32, ARCH_RISCV64, ARCH_RISCV32))]
    fn generic_sp(&self) -> &Word {
        self.inner().sp()
    }

    #[sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32, ARCH_RISCV64, ARCH_RISCV32))]
    fn generic_sp_mut(&mut self) -> &mut Word {
        self.inner_mut().sp_mut()
    }

    #[sel4_cfg(ARCH_X86_64)]
    fn generic_pc(&self) -> &Word {
        self.inner().rip()
    }

    #[sel4_cfg(ARCH_X86_64)]
    fn generic_pc_mut(&mut self) -> &mut Word {
        self.inner_mut().rip_mut()
    }

    #[sel4_cfg(ARCH_X86_64)]
    fn generic_sp(&self) -> &Word {
        self.inner().rsp()
    }

    #[sel4_cfg(ARCH_X86_64)]
    fn generic_sp_mut(&mut self) -> &mut Word {
        self.inner_mut().rsp_mut()
    }

    fn advance_pc(&mut self) {
        *self.generic_pc_mut() += 4;
    }
}

self_impl!(UnknownSyscallExt, UnknownSyscall);

pub trait UnknownSyscallExt {
    inner_decls!(UnknownSyscall);

    #[sel4_cfg(ARCH_AARCH64)]
    fn c_param(&self, ix: usize) -> Word {
        match ix {
            0 => self.inner().x0(),
            1 => self.inner().x1(),
            2 => self.inner().x2(),
            3 => self.inner().x3(),
            4 => self.inner().x4(),
            5 => self.inner().x5(),
            6 => self.inner().x6(),
            7 => self.inner().x7(),
            _ => panic!(),
        }
    }

    #[sel4_cfg(ARCH_AARCH32)]
    fn c_param(&self, ix: usize) -> Word {
        match ix {
            0 => self.inner().r0(),
            1 => self.inner().r1(),
            2 => self.inner().r2(),
            3 => self.inner().r3(),
            4 => self.inner().r4(),
            5 => self.inner().r5(),
            6 => self.inner().r6(),
            7 => self.inner().r7(),
            _ => panic!(),
        }
    }

    #[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))]
    fn c_param(&self, ix: usize) -> Word {
        match ix {
            0 => self.inner().a0(),
            1 => self.inner().a1(),
            2 => self.inner().a2(),
            3 => self.inner().a3(),
            4 => self.inner().a4(),
            5 => self.inner().a5(),
            6 => self.inner().a6(),
            _ => panic!(),
        }
    }

    #[sel4_cfg(ARCH_X86_64)]
    fn c_param(&self, ix: usize) -> Word {
        match ix {
            0 => self.inner().rdi(),
            1 => self.inner().rsi(),
            2 => self.inner().rdx(),
            3 => self.inner().rcx(),
            4 => self.inner().r8(),
            5 => self.inner().r9(),
            _ => panic!(),
        }
    }
}

impl<'a> UnknownSyscallInIpcBufferExt<'a> for UnknownSyscallInIpcBuffer<'a> {
    fn inner(&self) -> &Self {
        self
    }

    fn inner_mut(&mut self) -> &mut Self {
        self
    }
}

pub trait UnknownSyscallInIpcBufferExt<'a>
where
    Self: 'a,
{
    // inner_decls!(UnknownSyscallInIpcBuffer);

    fn inner(&self) -> &UnknownSyscallInIpcBuffer<'a>;

    fn inner_mut(&mut self) -> &mut UnknownSyscallInIpcBuffer<'a>;

    fn generic_pc<'b>(&'b self) -> &'b Word
    where
        'a: 'b,
    {
        self.inner().fault_ip()
    }

    fn generic_pc_mut<'b>(&'b mut self) -> &'b mut Word
    where
        'a: 'b,
    {
        self.inner_mut().fault_ip_mut()
    }

    fn advance_pc(&mut self)
    where
        Self: 'a,
    {
        *self.generic_pc_mut() += 4
    }

    #[sel4_cfg(ARCH_AARCH64)]
    fn c_param<'b>(&'b self, ix: usize) -> &'b Word
    where
        'a: 'b,
    {
        match ix {
            0 => self.inner().x0(),
            1 => self.inner().x1(),
            2 => self.inner().x2(),
            3 => self.inner().x3(),
            4 => self.inner().x4(),
            5 => self.inner().x5(),
            6 => self.inner().x6(),
            7 => self.inner().x7(),
            _ => panic!(),
        }
    }

    #[sel4_cfg(ARCH_AARCH64)]
    fn c_param_mut<'b>(&'b mut self, ix: usize) -> &'b mut Word
    where
        'a: 'b,
    {
        match ix {
            0 => self.inner_mut().x0_mut(),
            1 => self.inner_mut().x1_mut(),
            2 => self.inner_mut().x2_mut(),
            3 => self.inner_mut().x3_mut(),
            4 => self.inner_mut().x4_mut(),
            5 => self.inner_mut().x5_mut(),
            6 => self.inner_mut().x6_mut(),
            7 => self.inner_mut().x7_mut(),
            _ => panic!(),
        }
    }
}

#[allow(dead_code)]
struct BitField {
    start: u8,
    length: u8,
}

#[allow(dead_code)]
impl BitField {
    const fn new(start: u8, length: u8) -> Self {
        Self { start, length }
    }

    const fn from_range(bits: Range<u8>) -> Self {
        Self::new(bits.start, bits.end - bits.start)
    }

    const fn from_range_inclusive(bits: RangeInclusive<u8>) -> Self {
        Self::from_range(*bits.start()..*bits.end() + 1)
    }

    const fn bit(ix: u8) -> Self {
        Self::new(ix, 1)
    }

    const fn get(&self, word: Word) -> Word {
        (word >> self.start) & ((1 << self.length) - 1)
    }
}
