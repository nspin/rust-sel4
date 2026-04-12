//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_microkit::{Channel, ChannelSet, Handler, Infallible, debug_println};

const SERVER: Channel = Channel::new(0);

fn init() -> impl Handler {
    SERVER.notify();
    HandlerImpl {}
}

struct HandlerImpl {}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, _channels: ChannelSet) -> Result<(), Self::Error> {
        debug_println!("TEST_PASS");
        Ok(())
    }
}
