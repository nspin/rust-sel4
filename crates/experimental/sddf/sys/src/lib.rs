//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::useless_transmute)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// TODO name union upstream
pub type gpu_req_union_t = gpu_req__bindgen_ty_1;
