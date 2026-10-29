//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

sel4_config::cfg_select! {
    all(ARCH_ARM, PLAT_QEMU_ARM_VIRT) => {
        #[path = "qemu_arm_virt/mod.rs"]
        mod imp;
    }
    all(ARCH_ARM, PLAT_BCM2711) => {
        #[path = "bcm2711/mod.rs"]
        mod imp;
    }
    all(ARCH_RISCV, any(PLAT_SPIKE, PLAT_QEMU_RISCV_VIRT, PLAT_HIFIVE)) => {
        #[path = "riscv_generic/mod.rs"]
        mod imp;
    }
    _ => {}
}

#[allow(unused_imports)]
pub(crate) use imp::*;

pub(crate) trait Plat {
    fn init() {}

    fn init_per_core() {}

    fn put_char(c: u8);

    fn start_core(physical_core_id: usize, sp: usize);

    fn stop_core() -> ! {
        unimplemented!()
    }
}
