//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use core::ops::Range;
use core::ptr;
use core::slice;

use rkyv::Archive;
use rkyv::rancor::Error;

use sel4_capdl_initializer_core::{Initializer, InitializerBuffers, PerObjectBuffer};
use sel4_capdl_initializer_types::SpecForInitializer;
use sel4_immutable_cell::ImmutableCell;
use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
use sel4_root_task::{debug_print, root_task};

const LOG_LEVEL: LevelFilter = {
    // LevelFilter::Trace
    // LevelFilter::Debug
    LevelFilter::Info
};

static LOGGER: Logger = LoggerBuilder::const_default()
    .level_filter(LOG_LEVEL)
    .filter(|meta| meta.target() == "sel4_capdl_initializer_core")
    .write(|s| debug_print!("{}", s))
    .build();

#[root_task(stack_size = 0x10_000, heap_size = 0x10_000)]
fn main(bootinfo: &sel4::BootInfoPtr) -> ! {
    LOGGER.set().unwrap();
    let spec = get_spec();
    let mut buffers =
        InitializerBuffers::new(vec![PerObjectBuffer::const_default(); spec.objects.len()]);
    Initializer::initialize(
        bootinfo,
        user_image_bounds(),
        spec,
        *sel4_capdl_initializer_embedded_frames_data_start.get() as usize,
        &mut buffers,
    )
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static sel4_capdl_initializer_serialized_spec_data_start: ImmutableCell<*mut u8> =
    ImmutableCell::new(ptr::null_mut());

#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static sel4_capdl_initializer_serialized_spec_data_size: ImmutableCell<usize> =
    ImmutableCell::new(0);

#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static sel4_capdl_initializer_embedded_frames_data_start: ImmutableCell<*mut u8> =
    ImmutableCell::new(ptr::null_mut());

#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static sel4_capdl_initializer_image_start: ImmutableCell<*mut u8> =
    ImmutableCell::new(ptr::null_mut());

#[unsafe(no_mangle)]
#[unsafe(link_section = ".data")]
static sel4_capdl_initializer_image_end: ImmutableCell<*mut u8> =
    ImmutableCell::new(ptr::null_mut());

fn get_spec() -> &'static <SpecForInitializer as Archive>::Archived {
    let blob = unsafe {
        slice::from_raw_parts(
            *sel4_capdl_initializer_serialized_spec_data_start.get(),
            *sel4_capdl_initializer_serialized_spec_data_size.get(),
        )
    };
    rkyv::access::<_, Error>(blob).unwrap()
}

fn user_image_bounds() -> Range<usize> {
    (*sel4_capdl_initializer_image_start.get() as usize)
        ..(*sel4_capdl_initializer_image_end.get() as usize)
}
