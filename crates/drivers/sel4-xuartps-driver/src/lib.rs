#![no_std]

use core::convert::Infallible;

use embedded_hal_nb::nb;
use embedded_hal_nb::serial;

use sel4_driver_traits::HandleInterrupt;

mod device;

use device::Device;

pub struct Driver {
    device: Device,
}

impl Driver {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn new(ptr: *mut ()) -> Self {
        let device = Device::new(ptr.cast());
        device.init();
        Self { device }
    }
}

impl serial::ErrorType for Driver {
    type Error = Infallible;
}

impl serial::Read for Driver {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.device.get_char().ok_or(nb::Error::WouldBlock)
    }
}

impl serial::Write for Driver {
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.device.put_char(word);
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}

impl HandleInterrupt for Driver {
    fn handle_interrupt(&mut self) {
        self.device.clear_all_interrupts()
    }
}
