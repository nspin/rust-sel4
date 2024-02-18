//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use embedded_fatfs::{Date, DateTime, TimeProvider};

#[derive(Default, Debug)]
pub struct DummyTimeProvider(());

impl DummyTimeProvider {
    pub fn new() -> Self {
        Self(())
    }
}

impl TimeProvider for DummyTimeProvider {
    fn get_current_date(&self) -> Date {
        unimplemented!()
    }

    fn get_current_date_time(&self) -> DateTime {
        unimplemented!()
    }
}
