//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

//! sDDF components receive their configuration (shared memory region addresses, channel IDs,
//! etc.) via structures which are written into dedicated ELF sections after compilation. Use
//! [`config!`](crate::config!) to declare such a section and read its contents.

use core::cell::UnsafeCell;
use core::error::Error;
use core::fmt;
use core::mem;
use core::ptr;

pub trait Config: Copy {
    fn check_magic(&self) -> Result<(), MagicMismatchError>;
}

macro_rules! impl_config {
    ($check:path [ $($ty:ty),* $(,)? ]) => {
        $(
            impl $crate::config::Config for $ty {
                fn check_magic(&self) -> Result<(), $crate::config::MagicMismatchError> {
                    let mut copy = *self;
                    if unsafe { $check((&raw mut copy).cast()) } {
                        Ok(())
                    } else {
                        Err($crate::config::MagicMismatchError::new())
                    }
                }
            }
        )*
    };
}

pub(crate) use impl_config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagicMismatchError(());

impl MagicMismatchError {
    pub(crate) fn new() -> Self {
        Self(())
    }
}

impl fmt::Display for MagicMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "configuration magic mismatch")
    }
}

impl Error for MagicMismatchError {}

/// The contents are zero at compile time. Reads are volatile so that the compiler cannot assume
/// that the contents are still zero.
#[repr(transparent)]
pub struct ConfigSection<T>(UnsafeCell<T>);

unsafe impl<T> Sync for ConfigSection<T> {}

impl<T> ConfigSection<T> {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(UnsafeCell::new(unsafe { mem::zeroed() }))
    }

    fn read(&self) -> T
    where
        T: Copy,
    {
        unsafe { ptr::read_volatile(self.0.get()) }
    }
}

impl<T: Config> ConfigSection<T> {
    pub fn get(&self) -> Result<T, MagicMismatchError> {
        let config = self.read();
        config.check_magic()?;
        Ok(config)
    }
}

/// Declares a [`ConfigSection`] in the given ELF section and returns its contents, panicking if
/// the magic number does not match.
#[macro_export]
macro_rules! config {
    ($section:literal, $ty:ty) => {{
        #[unsafe(link_section = $section)]
        static CONFIG: $crate::config::ConfigSection<$ty> = $crate::config::ConfigSection::new();
        CONFIG
            .get()
            .unwrap_or_else(|_| panic!("magic mismatch in config section {}", $section))
    }};
}

#[macro_export]
macro_rules! device_resources {
    () => {
        $crate::resources::DeviceResources::new($crate::config!(
            ".device_resources",
            $crate::sys::device_resources_t
        ))
    };
}
