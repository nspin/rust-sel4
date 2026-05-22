//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;

use anyhow::Result;
use clap::Parser;
use object::elf::FileHeader64;
use object::read::elf::ElfFile;
use object::{Endianness, Object, ObjectSegment};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    loader: String,
    #[arg(long)]
    payload: String,
    #[arg(long, short = 'o')]
    out_file: String,
    #[arg(long, short = 'v')]
    verbose: bool,
}

const STACK_SIZE: usize = 4096;
const STACK_ALIGNMENT: usize = 16;
const PAYLOAD_ALIGNMENT: usize = WORD_SIZE_BYTES;
const WORD_SIZE_BYTES: usize = 8;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("{cli:#?}");
    }

    let loader_elf_bytes = fs::read(&cli.loader)?;
    let loader_elf = ElfFile::<FileHeader64<Endianness>>::parse(&loader_elf_bytes).unwrap();
    let loader_segment = loader_elf.segments().next().unwrap();
    assert_eq!(
        loader_segment.size(),
        loader_segment.data().unwrap().len().try_into().unwrap()
    );

    let payload_elf_bytes = fs::read(&cli.payload)?;
    let payload_elf = ElfFile::<FileHeader64<Endianness>>::parse(&payload_elf_bytes).unwrap();
    let mut payload = Payload::new(payload_elf.entry());
    for seg in payload_elf.segments() {
        payload.add_segment(seg);
    }

    let mut buf = loader_segment.data().unwrap().to_owned();
    buf.resize(buf.len().next_multiple_of(PAYLOAD_ALIGNMENT), 0);
    buf.extend_from_slice(&payload.serialize());

    let total_size = (buf.len() + STACK_SIZE).next_multiple_of(STACK_ALIGNMENT);

    buf[2 * WORD_SIZE_BYTES..][..WORD_SIZE_BYTES].copy_from_slice(&total_size.to_le_bytes());

    fs::write(&cli.out_file, &buf)?;

    Ok(())
}

struct Payload {
    entry: u64,
    regions: Vec<Region>,
    data: Vec<u8>,
}

struct Region {
    vaddr: u64,
    offset: u64,
    filesz: u64,
    memsz: u64,
}

impl Payload {
    fn new(entry: u64) -> Self {
        Self {
            entry,
            regions: vec![],
            data: vec![],
        }
    }

    fn add_segment<'a>(&mut self, seg: impl ObjectSegment<'a>) {
        let offset = self.data.len();
        let seg_data = seg.data().unwrap();
        self.data.extend_from_slice(seg_data);
        self.regions.push(Region {
            vaddr: seg.address(),
            offset: offset.try_into().unwrap(),
            filesz: seg_data.len().try_into().unwrap(),
            memsz: seg.size(),
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.extend_from_slice(&self.entry.to_be_bytes());
        buf.extend_from_slice(&u64::try_from(self.regions.len()).unwrap().to_be_bytes());
        for region in self.regions.iter() {
            buf.extend_from_slice(&region.vaddr.to_be_bytes());
            buf.extend_from_slice(&region.offset.to_be_bytes());
            buf.extend_from_slice(&region.filesz.to_be_bytes());
            buf.extend_from_slice(&region.memsz.to_be_bytes());
        }
        buf.extend_from_slice(&self.data);
        buf
    }
}
