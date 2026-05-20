//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::global_asm;

use sel4_config::sel4_cfg_bool;

pub(crate) fn start_core(physical_core_id: usize, sp: usize) {
    let start = psci_secondary_entry as *const PsciSecondaryEntryFn as usize;
    smccc::psci::cpu_on_32::<smccc::Smc>(
        physical_core_id.try_into().unwrap(),
        start.try_into().unwrap(),
        sp.try_into().unwrap(),
    )
    .unwrap();
}

type PsciSecondaryEntryFn = extern "C" fn() -> !;

unsafe extern "C" {
    fn psci_secondary_entry() -> !;
}

global_asm! {
    r#"
        .extern secondary_entry

        .section .text

        .global psci_secondary_entry
        psci_secondary_entry:
            mov sp, r0
            b secondary_entry
    "#
}
