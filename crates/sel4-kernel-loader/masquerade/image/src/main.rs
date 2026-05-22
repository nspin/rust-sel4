//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![feature(no_core)]

#![no_core]
#![no_std]
#![no_main]
#![feature(lang_items)]
#![allow(internal_features)]

// naked
//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//


static X: [0; 10] = [0; _];

#[lang = "sized"]
trait Sized {}
