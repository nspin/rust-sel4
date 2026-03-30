//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(core_intrinsics)]
#![allow(internal_features)]

use sel4_panicking_env::{debug_println, register_abort_trap};

const SUCCESS: char = '\u{0006}';
const FAILURE: char = '\u{0015}';

pub fn indicate_success() {
    debug_println!("INDICATE_SUCCESS\n{SUCCESS}\n");
}

fn indicate_failure() {
    debug_println!("INDICATE_FAILURE\n{FAILURE}\n");
}

register_abort_trap! {
    trap
}

fn trap() -> ! {
    indicate_failure();
    core::intrinsics::abort()
}

#[doc(hidden)]
#[macro_export]
macro_rules! embed_file {
    ($section_name:literal, $path:literal) => {
        const _: () = {
            #[used]
            #[unsafe(no_mangle)]
            #[unsafe(link_section = $section_name)]
            pub static DATA: [u8; include_bytes!($path).len()] = *include_bytes!($path);
        };
    }
}

#[macro_export]
macro_rules! embed_sdf_script {
    ($path:literal) => {
        $crate::embed_file!(".sdf_script", $path);
    }
}

#[macro_export]
macro_rules! embed_sdf_xml {
    ($path:literal) => {
        $crate::embed_file!(".sdf_xml", $path);
    }
}

#[macro_export]
macro_rules! embed_capdl_script {
    ($path:literal) => {
        $crate::embed_file!(".capdl_script", $path);
    }
}
