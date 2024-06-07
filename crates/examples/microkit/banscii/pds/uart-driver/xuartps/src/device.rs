use core::ops::Deref;

use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
use tock_registers::registers::{ReadOnly, ReadWrite};
use tock_registers::{register_bitfields, register_structs};

register_structs! {
    pub RegisterBlock {
        (0x00 => control: ReadWrite<u32, Control::Register>),
        (0x04 => mode: ReadWrite<u32, Mode::Register>),
        (0x08 => intrpt_en: ReadWrite<u32, Intrpts::Register>),
        (0x0C => intrpt_dis: ReadWrite<u32, Intrpts::Register>),
        (0x10 => intrpt_mask: ReadWrite<u32, Intrpts::Register>),
        (0x14 => chnl_int_sts: ReadWrite<u32, Intrpts::Register>),
        (0x18 => baud_rate_gen: ReadWrite<u32, Baud_rate_gen::Register>),
        (0x1C => rcvr_timeout: ReadWrite<u32, Rcvr_timeout::Register>),
        (0x20 => rcvr_fifo_trigger_level: ReadWrite<u32, Rcvr_FIFO_trigger_level::Register>),
        (0x24 => modem_ctrl: ReadWrite<u32, Modem_ctrl::Register>),
        (0x28 => modem_sts: ReadWrite<u32, Modem_sts::Register>),
        (0x2C => channel_sts: ReadOnly<u32, Channel_sts::Register>),
        (0x30 => tx_rx_fifo: ReadWrite<u8, TX_RX_FIFO::Register>),
        (0x31 => _reserved0),
        (0x34 => baud_rate_divider: ReadWrite<u32, Baud_rate_divider::Register>),
        (0x38 => flow_delay: ReadWrite<u32, Flow_delay::Register>),
        (0x3C => _reserved1),
        (0x44 => tx_fifo_trigger_level: ReadWrite<u32, Tx_FIFO_trigger_level::Register>),
        (0x48 => rx_fifo_byte_status: ReadWrite<u32, Rx_FIFO_byte_status::Register>),
        (0x4C => @END),
    }
}

register_bitfields! {
    u8,
    TX_RX_FIFO [
        FIFO OFFSET(0) NUMBITS(8) [],
    ],
}

register_bitfields! {
    u32,
    Control [
        STPBRK OFFSET(8) NUMBITS(1) [],
        STTBRK OFFSET(7) NUMBITS(1) [],
        RSTTO OFFSET(6) NUMBITS(1) [],
        TXDIS OFFSET(5) NUMBITS(1) [],
        TXEN OFFSET(4) NUMBITS(1) [],
        RXDIS OFFSET(3) NUMBITS(1) [],
        RXEN OFFSET(2) NUMBITS(1) [],
        TXRES OFFSET(1) NUMBITS(1) [],
        RXRES OFFSET(0) NUMBITS(1) [],
    ],
    Mode [
        WSIZE OFFSET(12) NUMBITS(2) [],
        CHMODE OFFSET(8) NUMBITS(2) [],
        NBSTOP OFFSET(6) NUMBITS(2) [],
        PAR OFFSET(3) NUMBITS(3) [],
        CHRL OFFSET(1) NUMBITS(2) [],
        CLKS OFFSET(0) NUMBITS(1) [],
    ],
    Intrpts [
        RBRK OFFSET(13) NUMBITS(1) [],
        TOVR OFFSET(12) NUMBITS(1) [],
        TNFUL OFFSET(11) NUMBITS(1) [],
        TTRIG OFFSET(10) NUMBITS(1) [],
        DMSI OFFSET(9) NUMBITS(1) [],
        TIMEOUT OFFSET(8) NUMBITS(1) [],
        PARE OFFSET(7) NUMBITS(1) [],
        FRAME OFFSET(6) NUMBITS(1) [],
        ROVR OFFSET(5) NUMBITS(1) [],
        TFUL OFFSET(4) NUMBITS(1) [],
        TEMPTY OFFSET(3) NUMBITS(1) [],
        RFUL OFFSET(2) NUMBITS(1) [],
        REMPTY OFFSET(1) NUMBITS(1) [],
        RTRIG OFFSET(0) NUMBITS(1) [],
    ],
    Baud_rate_gen [
        CD OFFSET(0) NUMBITS(16) [],
    ],
    Rcvr_timeout [
        RTO OFFSET(0) NUMBITS(8) [],
    ],
    Rcvr_FIFO_trigger_level [
        RTRIG OFFSET(0) NUMBITS(6) [],
    ],
    Modem_ctrl [
        FCM OFFSET(5) NUMBITS(1) [],
        RTS OFFSET(1) NUMBITS(1) [],
        DTR OFFSET(0) NUMBITS(1) [],
    ],
    Modem_sts [
        FCMS OFFSET(8) NUMBITS(1) [],
        DCD OFFSET(7) NUMBITS(1) [],
        RI OFFSET(6) NUMBITS(1) [],
        DSR OFFSET(5) NUMBITS(1) [],
        CTS OFFSET(4) NUMBITS(1) [],
        DDCD OFFSET(3) NUMBITS(1) [],
        TERI OFFSET(2) NUMBITS(1) [],
        DDSR OFFSET(1) NUMBITS(1) [],
        DCTS OFFSET(0) NUMBITS(1) [],
    ],
    Channel_sts [
        TNFUL OFFSET(14) NUMBITS(1) [],
        TTRIG OFFSET(13) NUMBITS(1) [],
        FDELT OFFSET(12) NUMBITS(1) [],
        TACTIVE OFFSET(11) NUMBITS(1) [],
        RACTIVE OFFSET(10) NUMBITS(1) [],
        TFUL OFFSET(4) NUMBITS(1) [],
        TEMPTY OFFSET(3) NUMBITS(1) [],
        RFUL OFFSET(2) NUMBITS(1) [],
        REMPTY OFFSET(1) NUMBITS(1) [],
        RTRIG OFFSET(0) NUMBITS(1) [],
    ],
    Baud_rate_divider [
        BDIV OFFSET(0) NUMBITS(8) [],
    ],
    Flow_delay [
        FDEL OFFSET(0) NUMBITS(6) [],
    ],
    Tx_FIFO_trigger_level [
        TTRIG OFFSET(0) NUMBITS(6) [],
    ],
    Rx_FIFO_byte_status [
        BYTE3_BREAK OFFSET(11) NUMBITS(1) [],
        BYTE3_FRM_ERR OFFSET(10) NUMBITS(1) [],
        BYTE3_PAR_ERR OFFSET(9) NUMBITS(1) [],
        BYTE2_BREAK OFFSET(8) NUMBITS(1) [],
        BYTE2_FRM_ERR OFFSET(7) NUMBITS(1) [],
        BYTE2_PAR_ERR OFFSET(6) NUMBITS(1) [],
        BYTE1_BREAK OFFSET(5) NUMBITS(1) [],
        BYTE1_FRM_ERR OFFSET(4) NUMBITS(1) [],
        BYTE1_PAR_ERR OFFSET(3) NUMBITS(1) [],
        BYTE0_BREAK OFFSET(2) NUMBITS(1) [],
        BYTE0_FRM_ERR OFFSET(1) NUMBITS(1) [],
        BYTE0_PAR_ERR OFFSET(0) NUMBITS(1) [],
    ],
}

pub struct Device {
    ptr: *mut RegisterBlock,
}

impl Device {
    pub unsafe fn new(ptr: *mut RegisterBlock) -> Self {
        Self { ptr }
    }

    fn ptr(&self) -> *mut RegisterBlock {
        self.ptr
    }

    pub fn init(&self) {
        self.rcvr_fifo_trigger_level
            .write(Rcvr_FIFO_trigger_level::RTRIG.val(1));
        self.intrpt_en.modify(Intrpts::RTRIG::SET);
        self.intrpt_dis.modify(Intrpts::RTRIG::CLEAR);
        // TODO: Check interrupt mask is correct now
        self.reset_paths();
        self.enable_tx();
        self.enable_rx();
        self.control.modify(Control::RSTTO::SET);
        self.control.modify(Control::STTBRK::CLEAR);
        self.control.modify(Control::STPBRK::SET);
        self.rcvr_timeout.set(0);
    }

    pub fn put_char(&self, c: u8) {
        self.tx_rx_fifo.write(TX_RX_FIFO::FIFO.val(c));
        if c == b'\n' {
            self.tx_rx_fifo.write(TX_RX_FIFO::FIFO.val(b'\r'));
        }
        while self.channel_sts.matches_all(Channel_sts::TEMPTY::CLEAR) {
            core::hint::spin_loop();
        }
    }

    pub fn clear_all_interrupts(&self) {
        self.chnl_int_sts.set(0xFFFFFFFF);
    }

    pub fn get_char(&self) -> Option<u8> {
        if self.channel_sts.matches_all(Channel_sts::REMPTY::CLEAR) {
            Some(self.tx_rx_fifo.read(TX_RX_FIFO::FIFO))
        } else {
            None
        }
    }

    fn reset_paths(&self) {
        self.control.modify(Control::TXRES::SET);
        self.control.modify(Control::RXRES::SET);
    }

    fn enable_tx(&self) {
        self.control.modify(Control::TXDIS::CLEAR);
        self.control.modify(Control::TXEN::SET);
    }

    fn enable_rx(&self) {
        self.control.modify(Control::RXDIS::CLEAR);
        self.control.modify(Control::RXEN::SET);
    }
}

impl Deref for Device {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}
