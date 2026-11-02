//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

//! Clients make protected procedure calls to the driver. [`GET_TIME`] returns the current time in
//! nanoseconds in the first message register. [`SET_TIMEOUT`] takes a timeout, relative to the
//! current time, in nanoseconds in the first message register. The driver notifies the client
//! when the timeout expires.

use crate::config::impl_config;
use crate::sys;

pub const GET_TIME: u32 = sys::SDDF_TIMER_GET_TIME;
pub const SET_TIMEOUT: u32 = sys::SDDF_TIMER_SET_TIMEOUT;

impl_config! {
    sys::timer_config_check_magic [
        sys::timer_client_config_t,
    ]
}

/// The sDDF client library accesses the IPC buffer through libsel4's `__sel4_ipc_buffer`, so
/// protection domains using this type must expose that symbol as a non-thread-local global (see
/// the `exposed-state` and `non-thread-local-state` features of the `sel4` crate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    channel: u32,
}

impl Client {
    pub fn from_config(config: &sys::timer_client_config_t) -> Self {
        Self {
            channel: config.driver_id.into(),
        }
    }

    pub fn time_now(&self) -> u64 {
        unsafe { sys::sddf_timer_time_now(self.channel) }
    }

    pub fn set_timeout(&self, timeout: u64) {
        unsafe { sys::sddf_timer_set_timeout(self.channel, timeout) }
    }
}
