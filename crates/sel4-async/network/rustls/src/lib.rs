//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

mod async_io;
mod error;
mod no_server_cert_verifier;
mod utils;

pub use async_io::{Connect, TcpConnector, TlsStream};
pub use error::Error;
pub use no_server_cert_verifier::NoServerCertVerifier;
