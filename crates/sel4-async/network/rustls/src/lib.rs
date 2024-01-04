//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

mod async_io;
mod no_server_cert_verifier;

pub use async_io::{Connect, Error, TcpConnector, TlsStream};
pub use no_server_cert_verifier::NoServerCertVerifier;
