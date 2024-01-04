//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(thread_local)]

extern crate alloc;

mod async_io;
mod compiler_builtins_supplement;
mod dummy_custom_getrandom;
mod error;
mod no_server_cert_verifier;
mod utils;

pub use async_io::{Connect, TcpConnector, TlsStream};
pub use dummy_custom_getrandom::seed_dummy_custom_getrandom;
pub use error::Error;
pub use no_server_cert_verifier::NoServerCertVerifier;
