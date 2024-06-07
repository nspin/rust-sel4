#![no_std]

use banscii_uart_driver_traits::UartDriver;

mod device;

use device::Device;

pub struct Driver {
    device: Device,
}

impl UartDriver for Driver {
    #[allow(clippy::missing_safety_doc)]
    unsafe fn new(ptr: *mut ()) -> Self {
        let device = Device::new(ptr.cast());
        device.init();
        Self { device }
    }

    fn put_char(&self, c: u8) {
        self.device.put_char(c)
    }

    fn get_char(&self) -> Option<u8> {
        self.device.get_char()
    }

    fn handle_interrupt(&self) {
        self.device.clear_all_interrupts()
    }
}
