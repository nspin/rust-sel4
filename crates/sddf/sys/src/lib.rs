//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![allow(non_camel_case_types)]

use sel4_sys::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type gpu_req_union_t = gpu_req__bindgen_ty_1;
