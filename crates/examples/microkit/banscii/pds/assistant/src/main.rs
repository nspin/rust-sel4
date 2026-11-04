//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Write;
use core::mem;
use core::str;

use embedded_hal_nb::nb;
use embedded_hal_nb::serial::{Read as _, Write as _};

use sddf_embedded_hal_nb::serial::Client as SerialClient;
use sel4_microkit::{
    Channel, ChannelSet, Handler, Infallible, memory_region_symbol, protection_domain, var,
};
use sel4_microkit_simple_ipc as simple_ipc;
use sel4_shared_memory::{
    SharedMemoryRef,
    access::{ReadOnly, ReadWrite},
};

use banscii_artist_interface_types as artist;
use banscii_assistant_core::Draft;

const MAX_SUBJECT_LEN: usize = 16;

#[protection_domain(heap_size = 0x10000)]
fn init() -> impl Handler {
    let serial_config = sddf::config!(".serial_client_config", sddf::sys::serial_client_config_t);
    let serial_client = unsafe { SerialClient::new(&serial_config) };
    let serial_rx = Channel::new(serial_client.rx_channel().unwrap());
    let serial_tx = Channel::new(serial_client.tx_channel());
    let artist = Channel::new(*var!(artist_channel_id: usize = usize::MAX));

    let region_in_size = *var!(region_in_size: usize = 0);
    let region_in = unsafe {
        SharedMemoryRef::new_read_only(
            memory_region_symbol!(region_in_start: *mut [u8], n = region_in_size),
        )
    };

    let region_out_size = *var!(region_out_size: usize = 0);
    let region_out = unsafe {
        SharedMemoryRef::new(
            memory_region_symbol!(region_out_start: *mut [u8], n = region_out_size),
        )
    };

    let mut this = HandlerImpl {
        serial_client,
        serial_rx,
        serial_tx,
        artist,
        region_in,
        region_out,
        buffer: Vec::new(),
    };

    this.prompt();
    this.flush();

    this
}

struct HandlerImpl {
    serial_client: SerialClient,
    serial_rx: Channel,
    serial_tx: Channel,
    artist: Channel,
    region_in: SharedMemoryRef<'static, [u8], ReadOnly>,
    region_out: SharedMemoryRef<'static, [u8], ReadWrite>,
    buffer: Vec<u8>,
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, channels: ChannelSet) -> Result<(), Self::Error> {
        let mut handled = false;
        if channels.contains(self.serial_tx) {
            // The driver has consumed from the Tx queue. Writes do not wait for this, so there is
            // nothing to do.
            handled = true;
        }
        if channels.contains(self.serial_rx) {
            handled = true;
            while let Ok(b) = self.serial_client.read() {
                if let b'\n' | b'\r' = b {
                    self.newline();
                    if !self.buffer.is_empty() {
                        self.try_create();
                    }
                    self.prompt();
                } else {
                    let c = char::from(b);
                    if c.is_ascii() && !c.is_ascii_control() {
                        if self.buffer.len() == MAX_SUBJECT_LEN {
                            writeln!(self.serial_client, "\n(char limit reached)").unwrap();
                            self.try_create();
                            self.prompt();
                        }
                        nb::block!(self.serial_client.write(b)).unwrap();
                        self.buffer.push(b);
                    }
                }
            }
        }
        assert!(handled, "unexpected channels: {}", channels.display());
        self.flush();
        Ok(())
    }
}

impl HandlerImpl {
    fn try_create(&mut self) {
        let mut buffer = Vec::new();
        mem::swap(&mut buffer, &mut self.buffer);
        match str::from_utf8(&buffer) {
            Ok(subject) => {
                self.create(subject);
            }
            Err(_) => {
                writeln!(self.serial_client, "error: input is not valid utf-8").unwrap();
            }
        };
        self.buffer.clear();
    }

    fn create(&mut self, subject: &str) {
        let draft = Draft::new(subject);

        let draft_start = 0;
        let draft_size = draft.pixel_data.len();
        let draft_end = draft_start + draft_size;

        self.region_out
            .as_mut_ptr()
            .index(draft_start..draft_end)
            .copy_from_slice(&draft.pixel_data);

        let req = artist::Request {
            height: draft.height,
            width: draft.width,
            draft_start,
            draft_size,
        };

        let resp: artist::Response = simple_ipc::call(self.artist, req).unwrap();

        let height = resp.height;
        let width = resp.width;

        let pixel_data = {
            let mut buf = vec![0; resp.masterpiece_size];
            self.region_in
                .as_ptr()
                .index(resp.masterpiece_start..resp.masterpiece_start + resp.masterpiece_size)
                .copy_into_slice(&mut buf);
            buf
        };

        let signature = {
            let mut buf = vec![0; resp.signature_size];
            self.region_in
                .as_ptr()
                .index(resp.signature_start..resp.signature_start + resp.signature_size)
                .copy_into_slice(&mut buf);
            buf
        };

        self.newline();

        for row in 0..height {
            for col in 0..width {
                let i = row * width + col;
                let b = pixel_data[i];
                nb::block!(self.serial_client.write(b)).unwrap();
            }
            self.newline();
        }

        self.newline();

        writeln!(self.serial_client, "Signature:").unwrap();
        for line in signature.chunks(32) {
            writeln!(self.serial_client, "{}", hex::encode(line)).unwrap();
        }

        self.newline();
    }

    fn prompt(&mut self) {
        write!(self.serial_client, "banscii> ").unwrap();
    }

    fn newline(&mut self) {
        writeln!(self.serial_client).unwrap();
    }

    fn flush(&mut self) {
        nb::block!(self.serial_client.flush()).unwrap();
    }
}
