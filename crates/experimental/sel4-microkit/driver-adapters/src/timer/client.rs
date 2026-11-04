//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::convert::Infallible;
use core::time::Duration;

use sddf::timer::Client as SddfClient;
use sel4_driver_interfaces::timer::{Clock, ErrorType, Timer};

/// See [`sddf::timer::Client`] for requirements on the protection domain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Client {
    inner: SddfClient,
}

impl Client {
    pub fn new(inner: SddfClient) -> Self {
        Self { inner }
    }
}

impl ErrorType for Client {
    type Error = Infallible;
}

impl Clock for Client {
    fn get_time(&mut self) -> Result<Duration, Self::Error> {
        Ok(Duration::from_nanos(self.inner.time_now()))
    }
}

impl Timer for Client {
    fn set_timeout(&mut self, relative: Duration) -> Result<(), Self::Error> {
        self.inner
            .set_timeout(relative.as_nanos().try_into().unwrap_or(u64::MAX));
        Ok(())
    }

    /// The sDDF timer protocol does not support clearing timeouts, so this is a no-op. The client
    /// may receive a spurious notification.
    fn clear_timeout(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
