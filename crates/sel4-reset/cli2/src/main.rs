//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;

use anyhow::{Error, ensure};
use num::NumCast;
use object::elf::{PF_W, PT_LOAD};
use object::read::elf::{ElfFile, FileHeader, ProgramHeader};
use object::{Endian, File, Object, ObjectSegment, ObjectSymbol};
use object::{Endianness, ObjectSection, ReadCache, ReadRef};
use rangemap::RangeSet;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    in_file_path: PathBuf,
    #[arg(short = 'o')]
    out_file_path: PathBuf,
    #[arg(long, short = 'v')]
    interactive: bool,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let in_bytes = fs::read(&cli.in_file_path)?;
    let in_file = File::parse(in_bytes.as_slice())?;

    let out_bytes = match in_file {
        File::Elf32(elf) => continue_with_type(&elf),
        File::Elf64(elf) => continue_with_type(&elf),
        _ => {
            panic!()
        }
    }?;

    fs::write(cli.out_file_path, &out_bytes)?;

    Ok(())
}

fn continue_with_type<'a, T>(orig_elf: &'a ElfFile<'a, T>) -> Result<Vec<u8>, Error>
where
    T: FileHeader<Word: NumCast + PatchValue>,
{
    Ok(vec![])
}

struct X<'a, T: FileHeader> {
    orig_elf: &'a ElfFile<'a, T>,
    data: Vec<u8>,
    region_meta: Vec<RegionMeta<T>>,
    new_phdrs: Vec<T::ProgramHeader>,
}

pub trait PatchValue {
    fn to_bytes(&self, endian: impl Endian) -> Vec<u8>;
}

impl PatchValue for u32 {
    fn to_bytes(&self, endian: impl Endian) -> Vec<u8> {
        endian.write_u32_bytes(*self).to_vec()
    }
}

impl PatchValue for u64 {
    fn to_bytes(&self, endian: impl Endian) -> Vec<u8> {
        endian.write_u64_bytes(*self).to_vec()
    }
}

struct RegionMeta<T: FileHeader> {
    dst_vaddr: T::Word,
    dst_size: T::Word,
    src_vaddr: T::Word,
    src_size: T::Word,
}

impl<T: FileHeader<Word: PatchValue>> RegionMeta<T> {
    fn pack(&self, endian: impl Endian, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.dst_vaddr.to_bytes(endian));
        buf.extend_from_slice(&self.dst_size.to_bytes(endian));
        buf.extend_from_slice(&self.src_vaddr.to_bytes(endian));
        buf.extend_from_slice(&self.src_size.to_bytes(endian));
    }
}

impl<'a, T: FileHeader<Word: NumCast + PatchValue>> X<'a, T> {
    fn new(orig_elf: &'a ElfFile<'a, T>) -> Self {
        Self {
            orig_elf,
            data: orig_elf.data().to_vec(),
            region_meta: vec![],
            new_phdrs: vec![],
        }
    }

    fn endian(&self) -> T::Endian {
        self.orig_elf.endian()
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn align(&mut self, align: usize) {
        self.data.resize(self.size().next_multiple_of(align), 0);
    }

    fn patch_word(&mut self, symbol_name: &str, value: T::Word) {
        let value_bytes = value.to_bytes(self.endian());
        let symbol = self.orig_elf.symbol_by_name(symbol_name).unwrap();
        let symbol_vaddr = symbol.address();
        assert_eq!(usize::try_from(symbol.size()).unwrap(), value_bytes.len());
        let offset_in_file = self
            .orig_elf
            .segments()
            .find_map(|segment| {
                let seg_mem_start = segment.address();
                let seg_mem_end = seg_mem_start + segment.size();
                if (seg_mem_start..seg_mem_end).contains(&symbol_vaddr) {
                    let offset_in_seg = symbol_vaddr - seg_mem_start;
                    let (seg_file_start, seg_file_size) = segment.file_range();
                    assert!(
                        offset_in_seg + u64::try_from(value_bytes.len()).unwrap() <= seg_file_size
                    );
                    Some(seg_file_start + offset_in_seg)
                } else {
                    None
                }
            })
            .unwrap();
        self.data[usize::try_from(offset_in_file).unwrap()..][..value_bytes.len()]
            .copy_from_slice(&value_bytes);
    }

    fn finalize(self) -> Vec<u8> {
        todo!()
    }
}
