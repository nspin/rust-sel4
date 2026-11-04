//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

//! Each client may have one outstanding timeout. Clients are identified by the channels on which
//! they make protected procedure calls.

use core::convert::Infallible;
use core::time::Duration;

use sddf::resources::DeviceResources;
use sddf::timer::{GET_TIME, SET_TIMEOUT};
use sel4_driver_interfaces::HandleInterrupt;
use sel4_driver_interfaces::timer::Timer;
use sel4_microkit::{Channel, ChannelSet, Handler, MessageInfo, get_mr, set_mr};

const MAX_CHANNELS: usize = 64;

#[derive(Clone, Debug)]
pub struct HandlerImpl<Driver> {
    driver: Driver,
    irq: Channel,
    timeouts: [Option<Duration>; MAX_CHANNELS],
}

impl<Driver: Timer> HandlerImpl<Driver> {
    /// The device's IRQ must be the first in `resources`.
    pub fn new(driver: Driver, resources: &DeviceResources) -> Self {
        Self {
            driver,
            irq: Channel::new(resources.irq(0)),
            timeouts: [None; MAX_CHANNELS],
        }
    }

    fn now(&mut self) -> Duration {
        self.driver.get_time().unwrap()
    }

    fn process_timeouts(&mut self) {
        let now = self.now();
        let mut next = None;
        for (i, timeout) in self.timeouts.iter_mut().enumerate() {
            if let Some(t) = *timeout {
                if t <= now {
                    Channel::new(i).notify();
                    *timeout = None;
                } else {
                    next = Some(next.map_or(t, |next: Duration| next.min(t)));
                }
            }
        }
        match next {
            Some(t) => self.driver.set_timeout(t - now).unwrap(),
            None => self.driver.clear_timeout().unwrap(),
        }
    }
}

impl<Driver: Timer + HandleInterrupt> Handler for HandlerImpl<Driver> {
    type Error = Infallible;

    fn notified(&mut self, channels: ChannelSet) -> Result<(), Self::Error> {
        if channels.contains(self.irq) {
            self.driver.handle_interrupt();
            self.irq.irq_ack().unwrap();
            self.process_timeouts();
        }
        Ok(())
    }

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        Ok(match u32::try_from(msg_info.label()) {
            Ok(GET_TIME) => {
                let now = self.now().as_nanos().try_into().unwrap();
                set_mr(0, now);
                MessageInfo::new(0, 1)
            }
            Ok(SET_TIMEOUT) if msg_info.count() >= 1 => {
                #[allow(clippy::useless_conversion)]
                let relative = Duration::from_nanos(get_mr(0).try_into().unwrap());
                let deadline = self.now().saturating_add(relative);
                self.timeouts[channel.index()] = Some(deadline);
                self.process_timeouts();
                MessageInfo::new(0, 0)
            }
            _ => panic!("unexpected request on channel {channel:?}: {msg_info:?}"),
        })
    }
}
